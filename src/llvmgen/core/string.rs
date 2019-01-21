use inkwell::values::BasicValueEnum;
use llvmgen::gen::FnContext;
use llvmgen::gen::Value;
use llvmgen::gen::convert;

use llvmgen::native;
use llvmgen::core;
use inkwell::AddressSpace;


fn get_str_len(native_string: &Value, context: &FnContext) -> Value {
    let native_string_ptr = match native_string {
        Value::Class(p, c) => *p,
        x => panic!("Expect Value::Class, found {:?}", x),
    };

    let first_param_pointer = unsafe { context.builder.build_struct_gep(native_string_ptr, 0, "first_param") };
    let raw_string_ptr = match context.builder.build_load(first_param_pointer, "load_raw") {
        BasicValueEnum::PointerValue(p) => p,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };

    native::string::get_str_len(raw_string_ptr, context)
}

pub fn instantiate_from_value(value: BasicValueEnum, context: &FnContext) -> Value {
    let native_string = native::string::instantiate_from_value(value, context);

    let instance_ptr = native::gen_malloc(&context.core.string_class.llvm_struct_type_ref.get().unwrap(), context);

    let first_param_pointer = unsafe { context.builder.build_struct_gep(instance_ptr, 0, "first_param_of_string") };
    context.builder.build_store(first_param_pointer, convert(&native_string));

    let second_param_pointer = unsafe { context.builder.build_struct_gep(instance_ptr, 1, "second_param_of_string") };
    context.builder.build_store(second_param_pointer, convert(&get_str_len(&native_string, context)));

    Value::Class(instance_ptr, context.core.string_class)
}
