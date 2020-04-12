use emit::{Emitter, Value};
use parse::tree::Method;
use inkwell::AddressSpace;
use inkwell::values::{FunctionValue, BasicValueEnum};
use emit::expr::ExprEmitter;
use emit::helper::Helper;
use inkwell::types::BasicTypeEnum;

pub trait EmitterMethod {
    fn apply_method(&self, method: &Method);
    fn create_llvm_main_method(&self, method: &Method);
}

impl EmitterMethod for Emitter<'_> {
    fn apply_method(&self, method: &Method) {
        let is_main = method.name.fragment == "main";
        let real_name = if is_main {
            "native__main"
        } else {
            method.name.fragment
        };

        let mut param_types = vec![];
        for param in &method.params {
            let param_class = unsafe { &*param.tpe.def_opt.get().unwrap() };
            param_types.push(param_class.llvm.get().unwrap().ptr_type(AddressSpace::Generic).into());
        }

        let return_type_class = unsafe { &*method.return_type.def_opt.get().unwrap() };
        let llvm_fn_type = if return_type_class.name.fragment == "Void" || return_type_class.name.fragment == "Native__Void" {
            self.context.void_type().fn_type(&param_types, false)
        } else {
            return_type_class.llvm.get().unwrap().ptr_type(AddressSpace::Generic).fn_type(&param_types, false)
        };

        let llvm_method = self.module.add_function(real_name, llvm_fn_type, None);
        method.llvm.set(Some(llvm_method));

        let first_block = self.context.append_basic_block(&llvm_method, "first_block");
        self.builder.position_at_end(&first_block);

        for (index, param) in method.params.iter().enumerate() {
            let param_class = unsafe { &*param.tpe.def_opt.get().unwrap() };
            let param_ptr_type = param_class.llvm.get().unwrap().ptr_type(AddressSpace::Generic);

            let alloca_ptr = self.builder.build_alloca(param_ptr_type, format!("Param {} of method {}", index, method.name.fragment).as_ref());
            self.builder.build_store(alloca_ptr, llvm_method.get_nth_param(index as u32).unwrap());
            param.llvm.set(Some(alloca_ptr));
        }

        let mut last_ret: Option<Value> = None;
        if method.name.fragment.starts_with("native__") {
            let mut native_params: Vec<BasicTypeEnum> = vec![];
            let mut native_args = vec![];
            for param in &method.params {
                let param_class = unsafe { &*param.tpe.def_opt.get().unwrap() };
                native_params.push(
                    match param_class.name.fragment {
                        "Native__Int" => self.context.i64_type().into(),
                        "Native__String" => self.context.i8_type().ptr_type(AddressSpace::Generic).into(),
                        other => panic!("Unrecognized {}", other),
                    }
                );

                let (arg_native_class_ptr, arg_native_class) = unwrap2!(Value::Class, self.read_ptr(param.llvm.get().unwrap(), param_class));
                let arg_native_value_ptr = unsafe { self.builder.build_struct_gep(arg_native_class_ptr, 0, "Gep for the native value") };
                native_args.push( self.builder.build_load(arg_native_value_ptr, "Load the native value"));
            }

            let native_method_name = &method.name.fragment["native__".len()..];
            let native_method = self.get_external_func(
                native_method_name,
                match return_type_class.name.fragment {
                    "Native__Void" => self.context.void_type().fn_type(&native_params, false),
                    other => panic!("Unrecognized {}", other),
                }
            );

            let native_ret_value = self.builder.build_call(native_method, &native_args, format!("Invoke native method {}", native_method_name).as_ref());
        } else {
            for (index, expr) in method.exprs.iter().enumerate() {
                let ret = self.apply_expr(expr);
                if index == (method.exprs.len() - 1) {
                    last_ret = Some(ret);
                }
            }
        }

        match return_type_class.name.fragment {
            "Void" | "Native__Void" => self.builder.build_return(None),
            _ => self.builder.build_return(Some(&self.convert(&last_ret.unwrap()))),
        };

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

        let ret_class = unsafe { &*method.return_type.def_opt.get().unwrap() };
        let ret_ptr = match llvm_ret.try_as_basic_value().left().unwrap() {
            BasicValueEnum::PointerValue(ptr) => ptr,
            x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
        };
        let first_arg_ptr = unsafe { self.builder.build_struct_gep(ret_ptr, 0, "Gep for the first param of Int") };
        let first_arg = match self.builder.build_load(first_arg_ptr, "Load the first param of Int") {
            BasicValueEnum::PointerValue(ptr) => ptr,
            x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
        };

        let native_int_first_arg_ptr = unsafe { self.builder.build_struct_gep(first_arg, 0, "Gep for the first param of Native__Int") };
        let int_value = match self.builder.build_load(native_int_first_arg_ptr, "Load the first param of Native__Int") {
            BasicValueEnum::IntValue(i) => i,
            x => panic!("Expect BasicValueEnum::IntValue, found {:?}", x),
        };

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
}
