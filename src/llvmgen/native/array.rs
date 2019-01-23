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
use llvmgen::native;
use inkwell::types::PointerType;


pub fn get_llvm_value_from_var(var: &tree::Var, context: &FnContext) -> PointerValue {
    let instance_ptr = match context.builder.build_load(var.llvm_ref.get().unwrap(), "load class instance") {
        BasicValueEnum::PointerValue(p) => p,
        _ => panic!()
    };

    get_llvm_value(instance_ptr.into(), context)
}

pub fn get_llvm_value(value: BasicValueEnum, context: &FnContext) -> PointerValue {
    let ptr= match value {
        BasicValueEnum::PointerValue(i) => i,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };
    let first_param_pointer = unsafe { context.builder.build_struct_gep(ptr, 0, "first_param_of_@Array") };
    match context.builder.build_load(first_param_pointer, "load_first_param_of_@Array") {
        BasicValueEnum::PointerValue(i) => i,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    }
}

pub fn instantiate_from_value(value: BasicValueEnum, context: &FnContext) -> Value {
    let ptr = match value {
        BasicValueEnum::PointerValue(p) => p,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };

    let instance_ptr = native::gen_malloc(&context.core.llvm_array_class.llvm_struct_type_ref.get().unwrap(), context);
    let first_param_pointer = unsafe { context.builder.build_struct_gep(instance_ptr, 0, "first_param_of_@Array") };

    // Array uses a generic pointer.
    let casted = context.builder.build_pointer_cast(ptr, context.context.i8_type().ptr_type(AddressSpace::Generic).ptr_type(AddressSpace::Generic), "casted");

    context.builder.build_store(first_param_pointer, casted);
    Value::Class(instance_ptr, context.core.llvm_array_class)
}
