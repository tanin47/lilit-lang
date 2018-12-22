use inkwell::AddressSpace;
use inkwell::types::BasicTypeEnum;
use inkwell::types::FunctionType;
use inkwell::types::StructType;
use inkwell::values::BasicValueEnum;
use inkwell::values::CallSiteValue;
use inkwell::values::FunctionValue;
use inkwell::module::Linkage;
use inkwell::values::PointerValue;

use llvmgen::gen;
use llvmgen::gen::FnContext;
use llvmgen::gen::Value;
use semantics::tree;

#[derive(Debug)]
pub enum NativeTypeEnum {
    I8,
    I32,
    String,
    Void
}

#[derive(Debug)]
pub enum NativeEnum {
    I8(I8),
    String(String)
}

impl NativeTypeEnum {
    pub fn get(name: &str) -> NativeTypeEnum {
        match name {
            "I8" => NativeTypeEnum::I8,
            "I32" => NativeTypeEnum::I32,
            "String" => NativeTypeEnum::String,
            "Void" => NativeTypeEnum::Void,
            x => panic!("Unrecognized LLVM type: {}", x),
        }
    }

    pub fn get_expr_type(&self) -> tree::ExprType {
        match self {
            NativeTypeEnum::I8 => tree::ExprType::Number,
            NativeTypeEnum::I32 => tree::ExprType::Number,
            NativeTypeEnum::String => tree::ExprType::String,
            NativeTypeEnum::Void => tree::ExprType::Void,
        }
    }

    fn get_func_type(&self, param_types: &[BasicTypeEnum], is_var_args: bool, context: &FnContext) -> FunctionType {
       match self {
           NativeTypeEnum::I8 => context.context.i8_type().fn_type(param_types, is_var_args),
           NativeTypeEnum::I32 => context.context.i32_type().fn_type(param_types, is_var_args),
           NativeTypeEnum::String => context.context.i8_type().ptr_type(AddressSpace::Generic).fn_type(param_types, is_var_args),
           NativeTypeEnum::Void => context.context.void_type().fn_type(param_types, is_var_args),
       }
    }

    fn convert_func_return_value(&self, value: &CallSiteValue, context: &FnContext) -> Value {
        match self {
            NativeTypeEnum::I8 | NativeTypeEnum::I32 => {
                match value.try_as_basic_value().left().unwrap() {
                    BasicValueEnum::IntValue(i) => Value::Number(i),
                    x => panic!("Expect BasicValueEnum, found {:?}", x),
                }
            },
            NativeTypeEnum::String => {
                match value.try_as_basic_value().left().unwrap() {
                    BasicValueEnum::PointerValue(p) => gen_string_from_cstring(p, context),
                    x => panic!("Expect BasicValueEnum, found {:?}", x),
                }
            },
            NativeTypeEnum::Void => Value::Void,
        }
    }
}

fn gen_string_from_cstring(
    cstring: PointerValue,
    context: &FnContext
) -> Value {
    let strlen = match context.module.get_function("strlen") {
        Some(f) => f,
        None => {
            let fn_type = context.context.i64_type().fn_type(
                &[
                    context.context.i8_type().ptr_type(AddressSpace::Generic).into()
                ],
                false);
            context.module.add_function("strlen", fn_type, Some(Linkage::External))
        },
    };
    let ret_strlen = context.builder.build_call(strlen, &[cstring.into()], "strlen");
    let cstring_size = match ret_strlen.try_as_basic_value().left().unwrap() {
        BasicValueEnum::IntValue(i) => i,
        _ => panic!("unable to get string's length")
    };

    let i8_type = context.context.i8_type();
    let i32_type = context.context.i32_type();

    let string = context.builder.build_alloca(context.core.string_struct_type, "string");

    let size_with_terminator = cstring_size.const_add(context.context.i32_type().const_int(1, false));
    let array = context.builder.build_array_alloca(i8_type, size_with_terminator,  "string_array");

    let memcpy = match context.module.get_function("llvm.memcpy.p0i8.p0i8.i64") {
        None => {
            context.module.add_function(
                "llvm.memcpy.p0i8.p0i8.i64",
                context.context.i64_type().fn_type(
                    &[
                        i8_type.ptr_type(AddressSpace::Generic).into(),
                        i8_type.ptr_type(AddressSpace::Generic).into(),
                        context.context.i64_type().into(),
                        context.context.i32_type().into(),
                        context.context.bool_type().into()
                    ],
                    false
                ),
                Some(Linkage::External)
            )
        }
        Some(f) => f,
    };

    context.builder.build_call(
        memcpy,
        &[
            array.into(),
            cstring.into(),
            size_with_terminator.into(),
            context.context.i32_type().const_int(4, false).into(),
            context.context.bool_type().const_zero().into()
        ],
        "memcpy"
    );

    let size_pointer = unsafe {
        context.builder.build_in_bounds_gep(
            string,
            &[i32_type.const_int(0, false), i32_type.const_int(0, false)],
            "gep"
        )
    };
    context.builder.build_store(size_pointer, cstring_size);

    let content_pointer = unsafe {
        context.builder.build_in_bounds_gep(
            string,
            &[i32_type.const_int(0, false), i32_type.const_int(1, false)],
            "gep"
        )
    };
    context.builder.build_store(content_pointer, array);

    Value::String(string)
}

fn get_external_func(
    name: &str,
    tpe: FunctionType,
    context: &FnContext,
) -> FunctionValue {
    match context.module.get_function(name) {
        Some(f) => f,
        None => {
            context.module.add_function(
                name,
                tpe,
                Some(Linkage::External)
            )
        },
    }
}

pub fn gen_invoke(
    invoke: &tree::LlvmInvoke,
    context: &FnContext,
) -> Value {
    let mut args: Vec<Value> = vec![];
    for arg in &invoke.args {
        args.push(gen::gen_expr(arg, context));
    }

    let mut args_types: Vec<BasicTypeEnum> = vec![];
    for arg in &args {
        args_types.push(NativeEnum::get_llvm_type(arg, context))
    }

    let llvm_func = get_external_func(
        &invoke.name,
        invoke.return_type.tpe.get_func_type(&args_types, invoke.is_varargs, context),
        context);

    let mut llvm_args: Vec<BasicValueEnum> = vec![];
    for arg in &args {
        llvm_args.push(NativeEnum::get_llvm_value(arg, context));
    }

    invoke.return_type.tpe.convert_func_return_value(&context.builder.build_call(llvm_func, &llvm_args, "function return value"), context)
}

pub fn instantiate(instance: &tree::LlvmClassInstance, context: &FnContext) -> Value {
    match &instance.class.tpe {
        NativeTypeEnum::I8 => I8::instantiate(instance, context),
        NativeTypeEnum::String => String::instantiate(instance, context),
        x => panic!("Unrecognized native class: {:?}", x),
    }
}

impl NativeEnum {
    pub fn get_ptr(&self) -> PointerValue {
       match self {
           NativeEnum::I8(i) => i.get_ptr(),
           NativeEnum::String(i) => i.get_ptr(),
       }
    }

    pub fn get_func_type(param_types: &[BasicTypeEnum], is_var_args: bool, context: &FnContext) -> FunctionType {
        context.context.i8_type().ptr_type(AddressSpace::Generic).fn_type(
            param_types,
            is_var_args
        )
    }

    pub fn get_llvm_type(value: &Value, context: &FnContext) -> BasicTypeEnum {
        let native_enum = match value {
            Value::LlvmClass(native_enum) => native_enum,
            x => panic!("Expect Value::LlvmClass, found {:?}", x),
        };

        match native_enum {
            NativeEnum::I8(i) => I8::get_llvm_type(context),
            NativeEnum::String(i) => String::get_llvm_type(context),
        }
    }

    pub fn get_llvm_value(value: &Value, context: &FnContext) -> BasicValueEnum {
        let native_enum = match value {
            Value::LlvmClass(native_enum) => native_enum,
            x => panic!("Expect Value::LlvmClass, found {:?}", x),
        };

        match native_enum {
            NativeEnum::I8(i) => i.get_llvm_value(context),
            NativeEnum::String(i) => i.get_llvm_value(context),
        }
    }
}

pub trait Native {
    fn get_ptr(&self) -> PointerValue;
    fn get_llvm_value(&self, context: &FnContext) -> BasicValueEnum;

    fn get_llvm_type(context: &FnContext) -> BasicTypeEnum;
    fn instantiate(instance: &tree::LlvmClassInstance, context: &FnContext) -> Value;
}

#[derive(Debug)]
pub struct I8 { ptr: PointerValue }
#[derive(Debug)]
pub struct String { ptr: PointerValue }

impl Native for String {
    fn get_ptr(&self) -> PointerValue {
        self.ptr
    }

    fn get_llvm_value(&self, context: &FnContext) -> BasicValueEnum {
        let string_content_pointer = unsafe {
            context.builder.build_in_bounds_gep(
                self.ptr,
                &[context.context.i32_type().const_int(0, false), context.context.i32_type().const_int(1, false)],
                "gep for content of @String")
        };
        context.builder.build_load(string_content_pointer, "load content of @String")
    }

    fn get_llvm_type(context: &FnContext) -> BasicTypeEnum {
        context.context.i8_type().ptr_type(AddressSpace::Generic).into()
    }

    fn instantiate(instance: &tree::LlvmClassInstance, context: &FnContext) -> Value {
        let string_ptr = match gen::gen_expr(&instance.expr, context) {
            Value::String(p) => p,
            x => panic!("Expect Value::String, Found {:?}", x),
        };

        Value::LlvmClass(NativeEnum::String(String { ptr: string_ptr }))
    }
}

impl Native for I8 {
    fn get_ptr(&self) -> PointerValue {
        return self.ptr;
    }

    fn get_llvm_value(&self, context: &FnContext) -> BasicValueEnum {
        context.builder.build_load(self.ptr, "load @I8")
    }

    fn get_llvm_type(context: &FnContext) -> BasicTypeEnum {
        context.context.i8_type().into()
    }

    fn instantiate(instance: &tree::LlvmClassInstance, context: &FnContext) -> Value {
        let value = gen::gen_expr(&instance.expr, context);
        let llvm_struct_ptr = context.builder.build_alloca(I8::get_llvm_type(context), "@I8");
        let first_param = unsafe {
            context.builder.build_in_bounds_gep(
                llvm_struct_ptr,
                &[context.context.i32_type().const_int(0, false), context.context.i32_type().const_int(0, false)],
                "gep for @I8")
        };
        context.builder.build_store(first_param, gen::convert(&value));

        Value::LlvmClass(NativeEnum::I8(I8 { ptr: llvm_struct_ptr }))
    }
}
