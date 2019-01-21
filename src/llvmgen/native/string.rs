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
use llvmgen::core;
use llvmgen::gen::FnContext;
use llvmgen::gen::Value;
use semantics::tree;
use llvmgen::native;
use inkwell::types::PointerType;
use inkwell::values::IntValue;

pub fn get_str_len(raw_string_ptr: PointerValue, context: &FnContext) -> Value {
    core::number::instantiate_from_value(get_str_len_as_int_value(raw_string_ptr, context).into(), context)
}

pub fn get_str_len_as_int_value(raw_string_ptr: PointerValue, context: &FnContext) -> IntValue {
    let strlen = native::get_external_func(
        "strlen",
        context.context.i64_type().fn_type(
            &[context.context.i8_type().ptr_type(AddressSpace::Generic).into()],
            false
        ),
        context
    );
    let ret_strlen = match context.builder.build_call(strlen, &[raw_string_ptr.into()], "strlen").try_as_basic_value().left().unwrap() {
        BasicValueEnum::IntValue(i) => i,
        _ => panic!("unable to get string's length")
    };
    context.builder.build_int_cast(ret_strlen, context.context.i32_type(), "string_size")
}

fn gen_string_from_cstring(
    cstring: PointerValue,
    context: &FnContext
) -> Value {
    let size = get_str_len_as_int_value(cstring, context);
    let size_with_terminator = context.builder.build_int_nsw_add(size, context.context.i32_type().const_int(1, false), "size_with_terminator");

    let array = native::gen_malloc_dynamic_array(&context.context.i8_type().into(), size_with_terminator, context);

    let strcpy = native::get_external_func(
        "strcpy",
        context.context.i8_type().ptr_type(AddressSpace::Generic).fn_type(
            &[
                context.context.i8_type().ptr_type(AddressSpace::Generic).into(),
                context.context.i8_type().ptr_type(AddressSpace::Generic).into(),
            ],
            false
        ),
        context
    );

    context.builder.build_call(
        strcpy,
        &[
            array.into(),
            cstring.into(),
        ],
        "strcpy"
    );

    Value::LlvmString(array)
}

pub fn get_llvm_value(ptr: PointerValue, context: &FnContext) -> BasicValueEnum {
    let string_content_pointer = unsafe { context.builder.build_struct_gep(ptr, 0, "gep_string") };
    match context.builder.build_load(string_content_pointer, "load_content") {
        BasicValueEnum::PointerValue(p) => BasicValueEnum::PointerValue(p),
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    }
}

pub fn get_llvm_type(context: &FnContext) -> PointerType {
    context.context.i8_type().ptr_type(AddressSpace::Generic)
}

pub fn instantiate_from_value(value: BasicValueEnum, context: &FnContext) -> Value {
    let ptr = match value {
        BasicValueEnum::PointerValue(ptr) => ptr,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };
    let string_pointer = match gen_string_from_cstring(ptr, context) {
        Value::LlvmString(p) => p,
        x => panic!("Expect Value::LlvmString, found {:?}", x),
    };

    let instance_ptr = native::gen_malloc(&context.core.llvm_string_class.llvm_struct_type_ref.get().unwrap(), context);
    let first_param_pointer = unsafe { context.builder.build_struct_gep(instance_ptr, 0, "first_param") };
    context.builder.build_store(first_param_pointer, string_pointer);
    Value::Class(instance_ptr, context.core.llvm_string_class)
}


pub fn instantiate(instance: &tree::ClassInstance, context: &FnContext) -> Value {
    let value = match gen::gen_expr(&instance.params[0], context) {
        Value::LlvmString(p) => Value::LlvmString(p),
        x => panic!("Expect Value::String, Found {:?}", x),
    };

    let class = match instance.tpe.get().unwrap() {
        tree::ExprType::Class(class_ptr) => {
            let class = unsafe { &*class_ptr };
            if class.name == "@String" {
                class
            } else {
                panic!("Expect @String, found {:?}", class)
            }
        }
        x => panic!("Expect a class, found {:?}", x),
    };

    instantiate_from_value(gen::convert(&value), context)
}
