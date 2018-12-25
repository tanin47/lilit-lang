use inkwell::AddressSpace;
use inkwell::module::Linkage;
use inkwell::types::BasicTypeEnum;
use inkwell::types::FunctionType;
use inkwell::types::StructType;
use inkwell::values::BasicValueEnum;
use inkwell::values::CallSiteValue;
use inkwell::values::FunctionValue;
use inkwell::values::PointerValue;

use llvmgen::gen;
use llvmgen::gen::FnContext;
use llvmgen::gen::Value;
use semantics::tree;
use inkwell::values::IntValue;

pub fn get_llvm_value_from_var(var: &tree::Var, context: &FnContext) -> IntValue {
    let instance_ptr = match context.builder.build_load(var.llvm_ref.get().unwrap(), "load class instance") {
        BasicValueEnum::PointerValue(p) => p,
        _ => panic!()
    };

    get_llvm_value(instance_ptr, context)
}

pub fn get_llvm_value(ptr: PointerValue, context: &FnContext) -> IntValue {
    let first_param_pointer = unsafe { context.builder.build_struct_gep(ptr, 0, "gep for the first param of @Boolean") };
    match context.builder.build_load(first_param_pointer, "load the first param of @Boolean") {
        BasicValueEnum::IntValue(i) => i,
        x => panic!("Expect BasicValueEnum::IntValue, found {:?}", x),
    }
}

pub fn get_llvm_type(context: &FnContext) -> BasicTypeEnum {
    context.context.bool_type().into()
}

pub fn instantiate_from_value(value: BasicValueEnum, class: &tree::Class, context: &FnContext) -> Value {
    match value {
        BasicValueEnum::IntValue(i) => (),
        x => panic!("Expect BasicValueEnum::IntValue, found {:?}", x),
    };

    let instance_ptr = context.builder.build_alloca(class.llvm_struct_type_ref.get().unwrap(), "@Boolean");
    let first_param_pointer = unsafe { context.builder.build_struct_gep(instance_ptr, 0, "first param of @Boolean") };
    context.builder.build_store(first_param_pointer, value);
    Value::Class(instance_ptr, class)
}

pub fn instantiate(instance: &tree::ClassInstance, context: &FnContext) -> Value {
    let value = match gen::gen_expr(&instance.params[0], context) {
        Value::LlvmBoolean(i) => Value::LlvmBoolean(i),
        x => panic!("Expect Value::LlvmBoolean, found {:?}", x),
    };

    let class = match instance.tpe.get().unwrap() {
        tree::ExprType::Class(class_ptr) => {
            let class = unsafe { &*class_ptr };
            if class.name == "@Boolean" {
                class
            } else {
                panic!("Expect @Boolean, found {:?}", class)
            }
        }
        x => panic!("Expect a class, found {:?}", x),
    };

    instantiate_from_value(gen::convert(&value), class, context)
}
