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

use llvmgen::native::string;
use llvmgen::native::int8;
use llvmgen::native::int32;

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
    let mut params: Vec<Value> = vec![];
    for arg in &invoke.args {
        params.push(gen::gen_expr(arg, context));
    }

    let mut param_types: Vec<BasicTypeEnum> = vec![];
    for param in &params {
        param_types.push(get_llvm_type(param, context))
    }

    let llvm_func = get_external_func(
        &invoke.name,
        match get_llvm_type_from_class(&invoke.return_type.get().unwrap(), context) {
            BasicTypeEnum::IntType(i) => i.fn_type(&param_types, invoke.is_varargs),
            BasicTypeEnum::IntType(i) => i.fn_type(&param_types, invoke.is_varargs),
            BasicTypeEnum::PointerType(i) => i.fn_type(&param_types, invoke.is_varargs),
            x => panic!("Unsupported function type: {:?}", x)
        },
        context);

    let mut llvm_params: Vec<BasicValueEnum> = vec![];
    for arg in &params {
        llvm_params.push(get_llvm_value(arg, context));
    }

    convert_func_return_value(&invoke.return_type.get().unwrap(), &context.builder.build_call(llvm_func, &llvm_params, "function return value"), context)
}

pub fn instantiate(instance: &tree::ClassInstance, context: &FnContext) -> Value {
    let class = match instance.tpe.get().unwrap() {
        tree::ExprType::Class(class) => unsafe { &*class },
        x => panic!("Expect a class, found {:?}", x),
    };
    match class.name.as_ref() {
        "@I8" => int8::instantiate(instance, context),
        "@I32" => int32::instantiate(instance, context),
        "@String" => string::instantiate(instance, context),
        x => panic!("Unrecognized native class: {:?}", x),
    }
}

pub fn convert_func_return_value(tpe: &tree::ExprType, value: &CallSiteValue, context: &FnContext) -> Value {
    let class = match tpe {
        tree::ExprType::Class(class) => unsafe { &**class },
        x => panic!("Expect a class, found {:?}", x),
    };

    match class.name.as_ref() {
        "@I8" => int8::instantiate_from_value(value.try_as_basic_value().left().unwrap(), class, context),
        "@I32" => int32::instantiate_from_value(value.try_as_basic_value().left().unwrap(), class, context),
        "@String" => string::instantiate_from_value(value.try_as_basic_value().left().unwrap(), class, context),
        x => panic!("Unrecognized LLVM class: {}", x),
    }
}

pub fn get_llvm_type_from_class(tpe: &tree::ExprType, context: &FnContext) -> BasicTypeEnum {
    let class = match tpe {
        tree::ExprType::Class(class) => unsafe { &**class },
        x => panic!("Expect a class, found {:?}", x),
    };

    match class.name.as_ref() {
        "@I8" => int8::get_llvm_type(context),
        "@I32" => int32::get_llvm_type(context),
        "@String" => string::get_llvm_type(context),
        x => panic!("Unrecognized LLVM class: {}", x),
    }
}

pub fn get_llvm_type(value: &Value, context: &FnContext) -> BasicTypeEnum {
    let native_class = match value {
        Value::Class(ptr, class) => unsafe { &**class },
        x => panic!("Expect Value::Class, found {:?}", x),
    };

    match native_class.name.as_ref() {
        "@I8" => int8::get_llvm_type(context),
        "@I32" => int32::get_llvm_type(context),
        "@String" => string::get_llvm_type(context),
        x => panic!("Unrecognized LLVM class: {}", x),
    }
}

pub fn get_llvm_value(value: &Value, context: &FnContext) -> BasicValueEnum {
    let (ptr, native_class) = match value {
        Value::Class(ptr, class) => (*ptr, unsafe { &**class }),
        x => panic!("Expect Value::Class, found {:?}", x),
    };

    match native_class.name.as_ref() {
        "@I8" => int8::get_llvm_value(ptr, context).into(),
        "@I32" => int32::get_llvm_value(ptr, context),
        "@String" => string::get_llvm_value(ptr, context),
        x => panic!("Unrecognized LLVM class: {}", x),
    }
}
