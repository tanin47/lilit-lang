use inkwell::values::BasicValueEnum;
use llvmgen::gen::FnContext;
use llvmgen::gen::Value;
use llvmgen::gen::convert;
use inkwell::IntPredicate;

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

pub fn get_char_array(instance: BasicValueEnum, context: &FnContext) -> Value {
    let instance_ptr = match instance {
        BasicValueEnum::PointerValue(p) => p,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };
    let first_param_pointer = unsafe { context.builder.build_struct_gep(instance_ptr, 0, "first_param") };
    let first_param = match context.builder.build_load(first_param_pointer, "load") {
        BasicValueEnum::PointerValue(p) => p,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };
    Value::Class(first_param, context.core.array_class)
}

pub fn get_llvm_string(instance: BasicValueEnum, context: &FnContext) -> Value {
    let array_instance = match get_char_array(instance, context) {
        Value::Class(p, c) => p,
        x => panic!("Expect Value::Class, found {:?}", x),
    };
    let array_size = native::int32::get_llvm_value(
        convert(&core::number::get_int32(convert(&core::array::get_size(array_instance.into(), context)), context)),
            context);
    let llvm_array = context.builder.build_pointer_cast(
    native::array::get_llvm_value(convert(&core::array::get_native(array_instance.into(), context)), context),
        context.core.char_class.llvm_struct_type_ref.get().unwrap().ptr_type(AddressSpace::Generic).ptr_type(AddressSpace::Generic),
        "llvm_array");
    let llvm_string = native::gen_malloc_dynamic_array(&context.context.i8_type().into(), array_size, context);

    let index_pointer = context.builder.build_alloca(context.context.i32_type(), "index_pointer");
    let cond_block = context.context.append_basic_block(context.func, "cond_block");
    let loop_body_block = context.context.append_basic_block(context.func, "loop_body_block");
    let after_loop_block = context.context.append_basic_block(context.func, "after_loop_block");

    context.builder.build_unconditional_branch(&cond_block);
    {
        context.builder.position_at_end(&cond_block);
        let index = match context.builder.build_load(index_pointer, "index") {
            BasicValueEnum::IntValue(i) => i,
            x => panic!("Expect BasicValueEnum::IntValue, found {:?}", x),
        };
        let cond_result = context.builder.build_int_compare(IntPredicate::SLT, index, array_size, "compare");
        context.builder.build_conditional_branch(cond_result, &loop_body_block, &after_loop_block);
    }

    {
        context.builder.position_at_end(&loop_body_block);
        let index = match context.builder.build_load(index_pointer, "index") {
            BasicValueEnum::IntValue(i) => i,
            x => panic!("Expect BasicValueEnum::IntValue, found {:?}", x),
        };
        let llvm_string_index_pointer = unsafe { context.builder.build_in_bounds_gep(llvm_string, &[index], "get_index") };
        let char_index_pointer = unsafe { context.builder.build_in_bounds_gep(llvm_array, &[index], "get_index") };

        let c = native::char::get_llvm_value(
            convert(&core::char::get_at_char(context.builder.build_load(char_index_pointer, "load_Char"), context)),
            context);
        context.builder.build_store(llvm_string_index_pointer, c);

        let next_index = context.builder.build_int_nsw_add(index, context.context.i32_type().const_int(1, false), "increment");
        context.builder.build_store(index_pointer, next_index);

        context.builder.build_unconditional_branch(&cond_block);
    }

    context.builder.position_at_end(&after_loop_block);
    Value::LlvmString(llvm_string)
}

pub fn instantiate_from_value(value: BasicValueEnum, context: &FnContext) -> Value {
    let raw_string_pointer = match value {
        BasicValueEnum::PointerValue(p) => p,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };
    let size = native::string::get_str_len_as_int_value(raw_string_pointer, context);

    let array = native::gen_malloc_dynamic_array(
        &context.core.char_class.llvm_struct_type_ref.get().unwrap().ptr_type(AddressSpace::Generic).into(),
        size,
        context);

    let index_pointer = context.builder.build_alloca(context.context.i32_type(), "index_pointer");
    let cond_block = context.context.append_basic_block(context.func, "cond_block");
    let loop_body_block = context.context.append_basic_block(context.func, "loop_body_block");
    let after_loop_block = context.context.append_basic_block(context.func, "after_loop_block");

    context.builder.build_unconditional_branch(&cond_block);

    {
        context.builder.position_at_end(&cond_block);
        let index = match context.builder.build_load(index_pointer, "index") {
            BasicValueEnum::IntValue(i) => i,
            x => panic!("Expect BasicValueEnum::IntValue, found {:?}", x),
        };
        let cond_result = context.builder.build_int_compare(IntPredicate::SLT, index, size, "compare");
        context.builder.build_conditional_branch(cond_result, &loop_body_block, &after_loop_block);
    }

    {
        context.builder.position_at_end(&loop_body_block);
        let index = match context.builder.build_load(index_pointer, "index") {
            BasicValueEnum::IntValue(i) => i,
            x => panic!("Expect BasicValueEnum::IntValue, found {:?}", x),
        };
        let c_index_pointer = unsafe { context.builder.build_in_bounds_gep(raw_string_pointer, &[index], "get_index") };
        let c = match context.builder.build_load(c_index_pointer, "char") {
            BasicValueEnum::IntValue(i) => i,
            x => panic!("Expect BasicValueEnum::IntValue, found {:?}", x),
        };
        let c_instance = core::char::instantiate_from_value(c.into(), context);

        let array_index_pointer = unsafe { context.builder.build_in_bounds_gep(array, &[index], "get_array_index") };
        context.builder.build_store(array_index_pointer, convert(&c_instance));

        let next_index = context.builder.build_int_nsw_add(index, context.context.i32_type().const_int(1, false), "increment");
        context.builder.build_store(index_pointer, next_index);

        context.builder.build_unconditional_branch(&cond_block);
    }

    context.builder.position_at_end(&after_loop_block);

    let instance_ptr = native::gen_malloc(&context.core.string_class.llvm_struct_type_ref.get().unwrap(), context);

    let first_param_pointer = unsafe { context.builder.build_struct_gep(instance_ptr, 0, "first_param_of_string") };
    let array_instance = core::array::instantiate_from_value(
        array.into(),
        core::number::instantiate_from_value(size.into(), context),
        core::number::instantiate_from_value(size.into(), context),
        context);
    context.builder.build_store(
        first_param_pointer,
        convert(&array_instance));

    Value::Class(instance_ptr, context.core.string_class)
}
