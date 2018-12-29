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
use inkwell::data_layout::DataLayout;
use inkwell::attributes::Attribute;
use inkwell::types::ArrayType;
use inkwell::types::BasicType;
use inkwell::types::IntType;
use inkwell::types::PointerType;
use inkwell::values::IntValue;
use inkwell::values::PointerMathValue;

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

pub fn gen_malloc_dynamic_array(tpe: &IntType, size: IntValue, context: &FnContext) -> PointerValue {
    let func_type = context.context
        .i8_type().ptr_type(AddressSpace::Generic)
        .fn_type(&[context.context.i64_type().into()], false);
    let func = get_external_func("GC_malloc", func_type, context);
    func.add_attribute(0, context.context.create_enum_attribute(Attribute::get_named_enum_kind_id("noalias"), 0));

    let p = match context.builder.build_call(func, &[tpe.size_of().const_mul(size).into()], "malloc").try_as_basic_value().left().unwrap() {
        BasicValueEnum::PointerValue(p) => p,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };
    gen_register_finalizer(p, context);

    context.builder.build_pointer_cast(p, tpe.ptr_type(AddressSpace::Generic), "cast")
}

pub fn gen_malloc_array(array_type: &ArrayType, context: &FnContext) -> PointerValue {
    let func_type = context.context
        .i8_type().ptr_type(AddressSpace::Generic)
        .fn_type(&[context.context.i64_type().into()], false);
    let func = get_external_func("GC_malloc", func_type, context);
    func.add_attribute(0, context.context.create_enum_attribute(Attribute::get_named_enum_kind_id("noalias"), 0));

    let p = match context.builder.build_call(func, &[array_type.size_of().unwrap().into()], "malloc").try_as_basic_value().left().unwrap() {
        BasicValueEnum::PointerValue(p) => p,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };
    gen_register_finalizer(p, context);

    context.builder.build_pointer_cast(p, array_type.ptr_type(AddressSpace::Generic), "cast")
}

pub fn gen_malloc(struct_type: &StructType, context: &FnContext) -> PointerValue {
    let func_type = context.context
        .i8_type().ptr_type(AddressSpace::Generic)
        .fn_type(&[context.context.i64_type().into()], false);
    let func = get_external_func("GC_malloc", func_type, context);
    func.add_attribute(0, context.context.create_enum_attribute(Attribute::get_named_enum_kind_id("noalias"), 0));

    let p = match context.builder.build_call(func, &[struct_type.size_of().unwrap().into()], "malloc").try_as_basic_value().left().unwrap() {
        BasicValueEnum::PointerValue(p) => p,
        x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
    };
    gen_register_finalizer(p, context);

    context.builder.build_pointer_cast(p, struct_type.ptr_type(AddressSpace::Generic), "cast")
}

pub fn gen_register_finalizer(ptr: PointerValue, context: &FnContext) {
    let finalizer_func = get_external_func(
        "finalizer",
        context.context.void_type().fn_type(
            &[
                context.context.i8_type().ptr_type(AddressSpace::Generic).into(),
                context.context.i8_type().ptr_type(AddressSpace::Generic).into()
            ],
            false
        ),
        context);

    let param_types = vec![
        context.context.i8_type().ptr_type(AddressSpace::Generic).into(),
        finalizer_func.as_global_value().as_pointer_value().get_type().into(),
        context.context.i8_type().ptr_type(AddressSpace::Generic).into(),
        finalizer_func.get_type().ptr_type(AddressSpace::Generic).into(),
        context.context.i8_type().ptr_type(AddressSpace::Generic).ptr_type(AddressSpace::Generic).into(),
    ];
    let func_type = context.context
        .void_type()
        .fn_type(&param_types, false);
    let func = get_external_func("GC_register_finalizer", func_type, context);

    context.builder.build_call(
        func,
        &[
            ptr.into(),
            finalizer_func.as_global_value().as_pointer_value().into(),
            context.context.i8_type().ptr_type(AddressSpace::Generic).const_null().into(),
            finalizer_func.get_type().ptr_type(AddressSpace::Generic).const_null().into(),
            context.context.i8_type().ptr_type(AddressSpace::Generic).ptr_type(AddressSpace::Generic).const_null().into(),
        ],
        "register_finalizer"
    );
}

pub fn gen_gc_init(context: &FnContext) {
    let func_type = context.context
        .void_type()
        .fn_type(&[], false);
    let func = get_external_func("GC_init", func_type, context);

    context.builder.build_call(func, &[], "gc_init");
}

pub fn gen_gc_collect(context: &FnContext) {
    let func_type = context.context
        .void_type()
        .fn_type(&[], false);
    let func = get_external_func("GC_gcollect", func_type, context);

    context.builder.build_call(func, &[], "gc_gcollect");
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
        "@Void" => int32::instantiate_from_value(context.context.i32_type().const_int(0, false).into(), context.core.llvm_number_class, context),
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
        "@Void" => context.context.i32_type().into(),
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
        "@I32" => int32::get_llvm_value(ptr, context).into(),
        "@String" => string::get_llvm_value(ptr, context),
        x => panic!("Unrecognized LLVM class: {}", x),
    }
}
