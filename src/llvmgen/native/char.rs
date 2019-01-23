use inkwell::values::BasicValueEnum;
use llvmgen::gen::FnContext;
use llvmgen::gen::Value;
use llvmgen::native;

pub fn instantiate_from_value(value: BasicValueEnum, context: &FnContext) -> Value {
    match value {
        BasicValueEnum::IntValue(i) => (),
        x => panic!("Expect BasicValueEnum::IntValue, found {:?}", x),
    };

    let instance_ptr = native::gen_malloc(&context.core.llvm_char_class.llvm_struct_type_ref.get().unwrap(), context);
    let first_param_pointer = unsafe { context.builder.build_struct_gep(instance_ptr, 0, "first_param_of_@Char") };
    context.builder.build_store(first_param_pointer, value);
    Value::Class(instance_ptr, context.core.llvm_char_class)
}

pub fn get_llvm_char(value: BasicValueEnum, context: &FnContext) -> Value {
    let instance_ptr = match value {
        BasicValueEnum::PointerValue(p) => p,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };

    let first_param_pointer = unsafe { context.builder.build_struct_gep(instance_ptr, 0, "first_param_of_@Char") };
    match context.builder.build_load(first_param_pointer, "load_llvm_char") {
        BasicValueEnum::IntValue(i) => Value::LlvmChar(i),
        x => panic!("Expect BasicValueEnum::IntValue, found {:?}", x),
    }
}
