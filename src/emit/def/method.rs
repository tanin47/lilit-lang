use emit::{Emitter, Value};
use parse::tree::Method;
use inkwell::AddressSpace;
use inkwell::IntPredicate;
use inkwell::values::{FunctionValue, BasicValueEnum, InstructionOpcode};
use emit::expr::ExprEmitter;
use emit::helper::Helper;
use inkwell::types::BasicTypeEnum;
use std::ptr::null;

pub trait EmitterMethod {
    fn apply_method(&self, method: &Method);
    fn create_llvm_main_method(&self, method: &Method);
    fn apply_native_method(&self, method: &Method);
}

impl EmitterMethod for Emitter<'_> {
    fn apply_method(&self, method: &Method) {
        if method.name.fragment.starts_with("native__") {
            self.apply_native_method(method);
            return;
        }

        let is_main = method.name.fragment == "main";
        let real_name = if is_main {
            "native__main".to_string()
        } else if let Some(parent_class) = method.parent_class {
            let parent_class = unsafe { &*parent_class };
            format!("lilit_user_space__{}__{}", parent_class.name.fragment, method.name.fragment)
        } else {
            format!("lilit_user_space__{}", method.name.fragment)
        };

        let mut param_types = vec![];

        for param in &method.params {
            param_types.push(param.get_llvm_type());
        }

        let return_type_class = unsafe { &*method.return_type.class_def.unwrap() };
        let llvm_fn_type = if return_type_class.name.fragment == "Void" {
            self.context.void_type().fn_type(&param_types, false)
        } else {
            return_type_class.llvm.get().unwrap().ptr_type(AddressSpace::Generic).fn_type(&param_types, false)
        };

        let llvm_method = self.module.add_function(&real_name, llvm_fn_type, None);
        method.llvm.set(Some(llvm_method));

        let first_block = self.context.append_basic_block(&llvm_method, "first_block");
        self.builder.position_at_end(&first_block);

        for (index, param) in method.params.iter().enumerate() {
            let alloca_ptr = self.builder.build_alloca(param.get_llvm_type(), format!("Param {} of method {}", index, method.name.fragment).as_ref());
            self.builder.build_store(alloca_ptr, llvm_method.get_nth_param(index as u32).unwrap());
            param.llvm.set(Some(alloca_ptr));
        }

        for (index, expr) in method.exprs.iter().enumerate() {
            let ret = self.apply_expr(expr);
            if index == (method.exprs.len() - 1) {
                match return_type_class.name.fragment {
                    "Void" => self.builder.build_return(None),
                    _ => self.builder.build_return(Some(&self.wrap_with_class(&ret, return_type_class)))
                };
            }
        }

        if !llvm_method.verify(true) {
            llvm_method.print_to_stderr();
            panic!("{}(...) is invalid.", method.name.fragment);
        }

        if is_main {
            self.create_llvm_main_method(method);
        }
    }

    fn create_llvm_main_method(&self, method: &Method) {
            let fn_type = self.context.i32_type().fn_type(
                &[
                    self.context.i32_type().into(),
                self.context.i8_type().ptr_type(AddressSpace::Generic).ptr_type(AddressSpace::Generic).into()
            ],
            false
        );

        let main = self.module.add_function("main", fn_type, None);
        let first_block = self.context.append_basic_block(&main, "first_block");
        self.builder.position_at_end(&first_block);

        let llvm_ret = self.builder.build_call(
            method.llvm.get().unwrap(),
            &[],
            &method.name.fragment);

        let return_type_class = unsafe { &*method.return_type.class_def.unwrap() };
        let ret_ptr = unwrap!(BasicValueEnum::PointerValue, llvm_ret.try_as_basic_value().left().unwrap());
        let first_arg_ptr = unsafe { self.builder.build_struct_gep(ret_ptr, 0, "Gep for the first param of Int") };
        let native_int = unwrap!(BasicValueEnum::PointerValue, self.builder.build_load(first_arg_ptr, "Load the first param of Int"));
        let native_int_first_arg_ptr = unsafe { self.builder.build_struct_gep(native_int, 0, "Gep for the first param of Native__Int") };
        let int_value = unwrap!(BasicValueEnum::IntValue, self.builder.build_load(native_int_first_arg_ptr, "Load the first param of Native__Int"));

        self.builder.build_return(Some(&self.builder.build_int_cast(
            int_value,
            self.context.i32_type(),
            "Cast return type"
        )));

        if !main.verify(true) {
            main.print_to_stderr();
            panic!("Native main(..) is invalid.");
        }
    }

    fn apply_native_method<'def>(&self, method: &Method<'def>) {
        let mut param_types = vec![];
        for param in &method.params {
            if param.is_varargs { continue; }

            let param_class = unsafe { &*param.tpe.class_def.unwrap() };
            param_types.push(self.get_type_for_native(param_class));
        }

        let return_type_class = unsafe { &*method.return_type.class_def.unwrap() };
        let is_varargs = method.params.last().map(|p|p.is_varargs).unwrap_or(false);
        let llvm_fn_type = match return_type_class.name.fragment {
            "Native__Void" => self.context.void_type().fn_type(&param_types, is_varargs),
            "Native__Int" => self.context.i64_type().fn_type(&param_types, is_varargs),
            "Native__Char" => self.context.i8_type().fn_type(&param_types, is_varargs),
            "Native__String" => self.context.i8_type().ptr_type(AddressSpace::Generic).fn_type(&param_types, is_varargs),
            other if other.starts_with("Native__Struct") => return_type_class.llvm_native.get().unwrap().ptr_type(AddressSpace::Generic).fn_type(&param_types, is_varargs),
            other => panic!("Unsupported {}", other)
        };

        let llvm_method = self.module.add_function(method.name.fragment, llvm_fn_type, None);
        method.llvm.set(Some(llvm_method));

        let first_block = self.context.append_basic_block(&llvm_method, "first_block");
        self.builder.position_at_end(&first_block);

        let mut native_params: Vec<BasicTypeEnum> = vec![];
        let mut native_args = vec![];
        for (index, param) in method.params.iter().enumerate() {
            if param.is_varargs { continue; }

            let param_class = unsafe { &*param.tpe.class_def.unwrap() };
            native_params.push(
                match param_class.name.fragment {
                    "Native__Int" => self.context.i64_type().into(),
                    "Native__String" => self.context.i8_type().ptr_type(AddressSpace::Generic).into(),
                    "Native__Char" => BasicTypeEnum::IntType(self.context.i8_type()),
                    other => panic!("Unrecognized {}", other),
                }
            );

            native_args.push(llvm_method.get_nth_param(index as u32).unwrap());
        }

        let mut va_list_ptr = None;
        if method.params.last().map(|p|p.is_varargs).unwrap_or(false) {
            va_list_ptr = Some(self.builder.build_alloca(self.va_list_struct_type, "va_list"));
            let varargs_param = method.params.last().unwrap();
            let va_start = self.get_external_func(
                "llvm.va_start",
                self.context.void_type().fn_type(&vec![BasicTypeEnum::PointerType(self.context.i8_type().ptr_type(AddressSpace::Generic))], false),
            );
            self.builder.build_call(va_start, vec![BasicValueEnum::PointerValue(va_list_ptr.unwrap())].as_ref(), "va_start(va_list)");

            native_params.push(BasicTypeEnum::PointerType(self.va_list_struct_type.ptr_type(AddressSpace::Generic)));
            native_args.push(BasicValueEnum::PointerValue(va_list_ptr.unwrap()));
        }

        let return_type_class = unsafe { &*method.return_type.class_def.unwrap() };
        let native_method_name = &method.name.fragment["native__".len()..];
        let native_method = self.get_external_func(
            native_method_name,
            match return_type_class.name.fragment {
                "Native__Void" => self.context.void_type().fn_type(&native_params, false),
                "Native__Int" => self.context.i64_type().fn_type(&native_params, false),
                "Native__Char" => self.context.i8_type().fn_type(&native_params, false),
                other if other.starts_with("Native__Struct") => return_type_class.llvm_native.get().unwrap().ptr_type(AddressSpace::Generic).fn_type(&native_params, false),
                other => panic!("Unrecognized {}", other),
            }
        );

        let native_ret_value = self.builder.build_call(
            native_method,
            native_args.as_ref(),
            format!("Invoke native method {}", native_method_name).as_ref()
        );

        if let Some(va_list_ptr) = va_list_ptr {
            let va_end = self.get_external_func(
                "llvm.va_end",
                self.context.void_type().fn_type(&vec![BasicTypeEnum::PointerType(self.context.i8_type().ptr_type(AddressSpace::Generic))], false),
            );
            self.builder.build_call(va_end, vec![BasicValueEnum::PointerValue(va_list_ptr)].as_ref(), "va_end(va_list)");
        }

        if return_type_class.name.fragment == "Native__Void" {
            self.builder.build_return(None);
        } else {
            self.builder.build_return(Some(&native_ret_value.try_as_basic_value().left().unwrap()));
        }
    }
}
