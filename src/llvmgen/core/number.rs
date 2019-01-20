use inkwell::values::BasicValueEnum;
use llvmgen::gen::FnContext;
use llvmgen::gen::Value;

use llvmgen::native;

pub fn instantiate_from_value(value: BasicValueEnum, context: &FnContext) -> Value {
    let i32_ptr = match native::int32::instantiate_from_value(value, context) {
        Value::Class(p, c) => p,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };

    let instance_ptr = native::gen_malloc(&context.core.number_class.llvm_struct_type_ref.get().unwrap(), context);
    let first_param_pointer = unsafe { context.builder.build_struct_gep(instance_ptr, 0, "first_param_of_i32") };
    context.builder.build_store(first_param_pointer, i32_ptr);
    Value::Class(instance_ptr, context.core.number_class)
}
