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


pub fn get_llvm_value(ptr: PointerValue, context: &FnContext) -> BasicValueEnum {
    let first_param_pointer = unsafe {
        context.builder.build_in_bounds_gep(
            ptr,
            &[context.context.i32_type().const_int(0, false), context.context.i32_type().const_int(0, false)],
            "gep for the first param of @String")
    };
    let first_param = match context.builder.build_load(first_param_pointer, "load the first param of @String") {
        BasicValueEnum::PointerValue(p) => p,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };
    let string_content_pointer = unsafe {
        context.builder.build_in_bounds_gep(
            first_param,
            &[context.context.i32_type().const_int(0, false), context.context.i32_type().const_int(1, false)],
            "gep for the content of the @String's first param")
    };
    match context.builder.build_load(string_content_pointer, "load the content of the @String's first param") {
        BasicValueEnum::PointerValue(p) => BasicValueEnum::PointerValue(p),
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    }
}

pub fn get_llvm_type(context: &FnContext) -> BasicTypeEnum {
    context.context.i8_type().ptr_type(AddressSpace::Generic).into()
}

pub fn instantiate_from_value(value: BasicValueEnum, class: &tree::Class, context: &FnContext) -> Value {
    let ptr = match value {
        BasicValueEnum::PointerValue(ptr) => ptr,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };
    let string_pointer = match gen_string_from_cstring(ptr, context) {
        Value::String(p) => p,
        x => panic!("Expect Value::String, found {:?}", x),
    };

    let instance_ptr = context.builder.build_alloca(class.llvm_struct_type_ref.get().unwrap(), "@String");
    let first_param_pointer = unsafe {
        context.builder.build_in_bounds_gep(
            instance_ptr,
            &[context.context.i32_type().const_int(0, false), context.context.i32_type().const_int(0, false)],
            "first param of @String")
    };
    context.builder.build_store(ptr, string_pointer);
    Value::Class(instance_ptr, class)
}


pub fn instantiate(instance: &tree::ClassInstance, context: &FnContext) -> Value {
    let string_ptr = match gen::gen_expr(&instance.params[0], context) {
        Value::String(p) => p,
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

    let instance_ptr = context.builder.build_alloca(class.llvm_struct_type_ref.get().unwrap(), "@String");
    let first_param_pointer = unsafe {
        context.builder.build_in_bounds_gep(
            instance_ptr,
            &[context.context.i32_type().const_int(0, false), context.context.i32_type().const_int(0, false)],
            "first param of @String")
    };
    context.builder.build_store(first_param_pointer, string_ptr);
    Value::Class(instance_ptr, class)
}
