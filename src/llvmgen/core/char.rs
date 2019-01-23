use inkwell::values::BasicValueEnum;
use llvmgen::gen::FnContext;
use llvmgen::gen::Value;

use llvmgen::native;

pub fn instantiate_from_value(value: BasicValueEnum, context: &FnContext) -> Value {
    let char_ptr = match native::char::instantiate_from_value(value, context) {
        Value::Class(p, c) => p,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };

    let instance_ptr = native::gen_malloc(&context.core.char_class.llvm_struct_type_ref.get().unwrap(), context);
    let first_param_pointer = unsafe { context.builder.build_struct_gep(instance_ptr, 0, "first_param_of_Char") };
    context.builder.build_store(first_param_pointer, char_ptr);
    Value::Class(instance_ptr, context.core.char_class)
}

pub fn get_at_char(value: BasicValueEnum, context: &FnContext) -> Value {
    let instance_ptr = match value {
        BasicValueEnum::PointerValue(p) => p,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };

    let first_param_pointer = unsafe { context.builder.build_struct_gep(instance_ptr, 0, "first_param_of_Char") };
    match context.builder.build_load(first_param_pointer, "load_at_char") {
        BasicValueEnum::PointerValue(p) => Value::Class(p, context.core.llvm_char_class),
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    }
}
