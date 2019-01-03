use inkwell::AddressSpace;
use inkwell::IntPredicate;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::OptimizationLevel;
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::types::ArrayType;
use inkwell::types::BasicTypeEnum;
use inkwell::types::StructType;
use inkwell::values::{BasicValue, FunctionValue, IntValue};
use inkwell::values::BasicValueEnum;
use inkwell::values::PointerValue;
use inkwell::values::StructValue;
use inkwell::values::VectorValue;

use std::cell::Cell;
use semantics::tree;
use inkwell::types::PointerType;
use inkwell::types::BasicType;
use llvmgen::native;
use inkwell::attributes::Attribute;

#[derive(Debug)]
pub enum Value {
    Void,
    LlvmNumber(IntValue),
    LlvmBoolean(IntValue),
    LlvmString(PointerValue),
    Class(PointerValue, *const tree::Class),
}

pub fn convert(value: &Value) -> BasicValueEnum {
    match value {
        Value::LlvmNumber(i) => (*i).into(),
        Value::LlvmBoolean(b) => (*b).into(),
        Value::LlvmString(p) => (*p).into(),
        Value::Class(p, c) => (*p).into(),
        Value::Void => panic!("can't convert void"),
    }
}

pub struct Core<'a> {
    pub string_struct_type: StructType,
    pub number_class: &'a tree::Class,
    pub llvm_number_class: &'a tree::Class,
    pub boolean_class: &'a tree::Class,
    pub llvm_boolean_class: &'a tree::Class,
    pub string_class: &'a tree::Class,
    pub llvm_string_class: &'a tree::Class,
}

struct ModContext<'a, 'b, 'c, 'd> {
    module: &'a Module,
    context: &'b Context,
    builder: &'c Builder,
    core: &'d Core<'d>,
}

pub struct FnContext<'a, 'b, 'c, 'd, 'e> {
    pub func: &'a FunctionValue,
    pub module: &'b Module,
    pub context: &'c Context,
    pub builder: &'d Builder,
    pub core: &'e Core<'e>,
}

pub fn generate(
    module: &tree::Mod,
    context: &Context,
    builder: &Builder,
) -> Module {
    let llvm_module = context.create_module("main");

    let core = Core {
        string_struct_type: StructType::struct_type(
            &[
                context.i32_type().into(),
                context.i8_type().ptr_type(AddressSpace::Generic).into()
            ],
            false
        ),
        number_class: unsafe { &*module.number_class.get().unwrap() },
        llvm_number_class: unsafe { &*module.llvm_number_class.get().unwrap() },
        boolean_class: unsafe { &*module.boolean_class.get().unwrap() },
        llvm_boolean_class: unsafe { &*module.llvm_boolean_class.get().unwrap() },
        string_class: unsafe { &*module.string_class.get().unwrap() },
        llvm_string_class: unsafe { &*module.llvm_string_class.get().unwrap() },
    };
    {
        let context = ModContext {
            module: &llvm_module,
            context,
            builder,
            core: &core
        };
        for unit in &module.units {
            match unit {
                tree::ModUnit::Class(ref class) => gen_class_opaque_struct(class, &context),
                _ => (),
            }
        }
        for unit in &module.units {
            match unit {
                tree::ModUnit::Class(ref class) => gen_class_struct(class, &context),
                _ => (),
            }
        }
        for unit in &module.units {
            match unit {
                tree::ModUnit::Class(ref class) => gen_class_methods(class, &context),
                _ => (),
            }
        }
        for unit in &module.units {
            gen_mod_unit(unit, &context);
        }
    }
    return llvm_module;
}

fn gen_mod_unit(
    unit: &tree::ModUnit,
    context: &ModContext,
) {
    match unit {
        tree::ModUnit::Func(ref func) => gen_func(func, context),
        _ => (),
    }
}

fn gen_class_opaque_struct(
    class: &tree::Class,
    context: &ModContext
) {
    let opaque_struct = context.context.opaque_struct_type(&class.name);
    class.llvm_struct_type_ref.set(Some(opaque_struct));
}

fn gen_class_struct(
    class: &tree::Class,
    context: &ModContext
) {
    let mut type_enums: Vec<BasicTypeEnum> = vec![];
    for param in &class.params {
        type_enums.push(match param.tpe.get().unwrap() {
            tree::ExprType::LlvmString => context.core.string_struct_type.ptr_type(AddressSpace::Generic).into(),
            tree::ExprType::LlvmBoolean => context.context.bool_type().into(),
            tree::ExprType::LlvmNumber => context.context.i32_type().into(),
            tree::ExprType::Class(class_ptr) => {
                let class = unsafe { &*class_ptr };
                class.llvm_struct_type_ref.get().unwrap().ptr_type(AddressSpace::Generic).into()
            },
            x => panic!()
        });
    }
    class.llvm_struct_type_ref.get().unwrap().set_body(&type_enums, false);
}

fn gen_class_methods(
    class: &tree::Class,
    context: &ModContext
) {
    for method in &class.methods {
       gen_method(method, class, context) ;
    }
}

fn gen_method(
    method: &tree::Func,
    class: &tree::Class,
    context: &ModContext,
) {
    let mut param_types: Vec<BasicTypeEnum> = vec![];
    for param in &method.params {
        param_types.push(match param.tpe.get().unwrap() {
            tree::ExprType::Class(class_ptr) => {
                let class = unsafe { &*class_ptr };
                class.llvm_struct_type_ref.get().unwrap().ptr_type(AddressSpace::Generic).into()
            },
            x => panic!("Unrecognized param: {:?}", x)
        });
    }

    let fn_type = match method.return_type.get().unwrap() {
        tree::ExprType::Void => context.context.void_type().fn_type(&param_types, false),
        tree::ExprType::Class(class_ptr) => {
            let class = unsafe { &*class_ptr };
            class.llvm_struct_type_ref.get().unwrap().ptr_type(AddressSpace::Generic).fn_type(&param_types, false)
        },
        x => panic!("Unsupported return type: {:?}", x)
    };

    let func_name = format!("__{}__{}", class.name, method.name);
    let function = context.module.add_function(&func_name, fn_type, None);
    method.llvm_ref.set(Some(function));

    let first_block = context.context.append_basic_block(&function, "first_block");
    context.builder.position_at_end(&first_block);

    for (index, param) in method.params.iter().enumerate() {
        let tpe: BasicTypeEnum = match param.tpe.get().unwrap() {
            tree::ExprType::Class(class_ptr) => {
                let class = unsafe { &*class_ptr };
                class.llvm_struct_type_ref.get().unwrap().ptr_type(AddressSpace::Generic).into()
            },
            _ => panic!()
        };
        let ptr = context.builder.build_alloca(tpe, "param");
        context.builder.build_store(ptr, function.get_nth_param(index as u32).unwrap());
        param.var.llvm_ref.set(Some(ptr));
    }

    let fn_context = FnContext {
        func: &function,
        module: context.module,
        context: context.context,
        builder: context.builder,
        core: context.core,
    };

    if class.is_llvm {
        if class.name == "@I32" {
            if method.name == "add" {
                let first = native::int32::get_llvm_value_from_var(&method.params[0].var, &fn_context);
                let second = native::int32::get_llvm_value_from_var(&method.params[1].var, &fn_context);
                let sum = context.builder.build_int_nsw_add(first, second, "@I32.add");
                let i8_value = native::int32::instantiate_from_value(sum.into(), &class, &fn_context);
                context.builder.build_return(Some(&convert(&i8_value)));
            } else if method.name == "is_greater_than" {
                let first = native::int32::get_llvm_value_from_var(&method.params[0].var, &fn_context);
                let second = native::int32::get_llvm_value_from_var(&method.params[1].var, &fn_context);

                let result = context.builder.build_int_compare(IntPredicate::SGT, first, second, "@I32.sgt");
                let boolean_value = native::boolean::instantiate_from_value(result.into(), context.core.llvm_boolean_class, &fn_context);
                context.builder.build_return(Some(&convert(&boolean_value)));
            } else if method.name == "to_num" {
                let number_ptr = native::gen_malloc(&context.core.number_class.llvm_struct_type_ref.get().unwrap(), &fn_context);
                let first_param_pointer = unsafe { context.builder.build_struct_gep(number_ptr, 0, "first param") };

                let i32_ptr = match context.builder.build_load(method.params[0].var.llvm_ref.get().unwrap(), "load i32") {
                    BasicValueEnum::PointerValue(p) => p,
                    _ => panic!()
                };
                context.builder.build_store(first_param_pointer, i32_ptr);
                context.builder.build_return(Some(&number_ptr));
            } else {
                panic!("Unsupported LLVM method: {}.{}", class.name, method.name);
            }
        } else if class.name == "@Boolean" {
            if method.name == "to_boolean" {
                let boolean_ptr = native::gen_malloc(&context.core.boolean_class.llvm_struct_type_ref.get().unwrap(), &fn_context);
                let first_param_pointer = unsafe { context.builder.build_struct_gep(boolean_ptr, 0, "gep for the first param of Number, which is @Boolean") };
                let at_boolean_ptr = match context.builder.build_load(method.params[0].var.llvm_ref.get().unwrap(), "load @Boolean") {
                    BasicValueEnum::PointerValue(p) => p,
                    _ => panic!()
                };
                context.builder.build_store(first_param_pointer, at_boolean_ptr);
                context.builder.build_return(Some(&boolean_ptr));
            } else {
                panic!("Unsupported LLVM method: {}.{}", class.name, method.name);
            }
        } else {
            panic!("Unsupported LLVM class: {}", class.name);
        }
    } else {
        for (index, expr) in method.exprs.iter().enumerate() {
            let ret = gen_expr(expr, &fn_context);
            if index == (method.exprs.len() - 1) {
                match method.return_type.get().unwrap() {
                    tree::ExprType::Void => (),
                    _ => { context.builder.build_return(Some(&convert(&ret))); },
                };
            }
        }
    }

    if !function.verify(true) {
        panic!("{}.{} is invalid.", class.name, method.name);
    }
}

fn gen_func(
    func: &tree::Func,
    context: &ModContext,
) {
    let mut param_types: Vec<BasicTypeEnum> = vec![];
    for param in &func.params {
       param_types.push(match param.tpe.get().unwrap() {
           tree::ExprType::Class(class_ptr) => {
               let class = unsafe { &*class_ptr };
               class.llvm_struct_type_ref.get().unwrap().ptr_type(AddressSpace::Generic).into()
           },
           x => panic!("Unrecognized param: {:?}", x),
       });
    }

    let fn_type = if func.name == "main" {
        context.context.i32_type().fn_type(&param_types, false)
    } else {
        match func.return_type.get().unwrap() {
            tree::ExprType::Void => context.context.void_type().fn_type(&param_types, false),
            tree::ExprType::Class(class_ptr) => {
                let class = unsafe { &*class_ptr };
                class.llvm_struct_type_ref.get().unwrap().ptr_type(AddressSpace::Generic).fn_type(&param_types, false)
            },
            x => panic!("Unsupported return type: {:?}", x),
        }
    };

    let function = context.module.add_function(&func.name, fn_type, None);
    func.llvm_ref.set(Some(function));

    let first_block = context.context.append_basic_block(&function, "first_block");
    context.builder.position_at_end(&first_block);

    for (index, param) in func.params.iter().enumerate() {
        let tpe: BasicTypeEnum = match param.tpe.get().unwrap() {
            tree::ExprType::Class(class_ptr) => {
                let class = unsafe { &*class_ptr };
                class.llvm_struct_type_ref.get().unwrap().ptr_type(AddressSpace::Generic).into()
            },
            _ => panic!()
        };
        let ptr = context.builder.build_alloca(tpe, "param");
        context.builder.build_store(ptr, function.get_nth_param(index as u32).unwrap());
        param.var.llvm_ref.set(Some(ptr));
    }

    let fn_context = FnContext {
        func: &function,
        module: context.module,
        context: context.context,
        builder: context.builder,
        core: context.core,
    };

    if func.name == "main" {
        native::gen_gc_init(&fn_context);
    }

    for (index, expr) in func.exprs.iter().enumerate() {
        let ret = gen_expr(expr, &fn_context);
        if index == (func.exprs.len() - 1) {
            let ret = if func.name == "main" {
                native::gen_gc_collect(&fn_context);

                let (number_ptr, number_class) = match ret {
                    Value::Class(ptr, class) => (ptr, unsafe { &*class }),
                    x => panic!("Expect Number, found {:?}", x),
                };
                let i32_ptr = unsafe {
                    context.builder.build_struct_gep(number_ptr, 0, "gep for @I32")
                };
                let i32_instance = match context.builder.build_load(i32_ptr, "load @I32") {
                    BasicValueEnum::PointerValue(ptr) => ptr,
                    x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
                };
                Value::LlvmNumber(native::int32::get_llvm_value(i32_instance, &fn_context))
            } else {
                ret
            };

            match func.return_type.get().unwrap() {
                tree::ExprType::Void => (),
                _ => { context.builder.build_return(Some(&convert(&ret))); },
            };
        }
    }

    if !function.verify(true) {
        panic!("{} is invalid.", func.name);
    }
}

pub fn gen_expr(
    expr: &tree::Expr,
    context: &FnContext
) -> Value {
    match expr {
        tree::Expr::Invoke(ref invoke) => gen_invoke(invoke, context),
        tree::Expr::DotInvoke(ref invoke) => gen_dot_invoke(invoke, context),
        tree::Expr::DotMember(ref member) => gen_dot_member(member, context),
        tree::Expr::LlvmInvoke(ref invoke) => native::gen_invoke(invoke, context),
        tree::Expr::Assignment(ref assignment) => gen_assignment(assignment, context),
        tree::Expr::ReadVar(ref read_var) => gen_read_var(read_var, context),
        tree::Expr::IfElse(ref if_else) => gen_if_else(if_else, context),
        tree::Expr::ClassInstance(ref class_instance) => gen_class_instance(class_instance, context),
        tree::Expr::LlvmClassInstance(ref class_instance) => gen_llvm_class_instance(class_instance, context),
        tree::Expr::LlvmNumber(ref number) => gen_llvm_number(number, context),
        tree::Expr::LlvmBoolean(ref boolean) => gen_llvm_boolean(boolean, context),
        tree::Expr::LlvmString(ref literal_string) => gen_llvm_string(literal_string, context),
    }
}

fn gen_dot_member(
    dot_member: &tree::DotMember,
    context: &FnContext
) -> Value {
    let expr = gen_expr(&dot_member.expr, context);
    let llvm_expr = match expr {
        Value::Class(ptr, class) => ptr,
        _ => panic!(),
    };

    let ptr = unsafe {
        context.builder.build_struct_gep(llvm_expr, dot_member.member.param_index.get().unwrap() as u32, "gep")
    };

    let llvm_ret = context.builder.build_load(ptr, &format!("load param {}", dot_member.member.name));

    match dot_member.tpe.get().unwrap() {
        tree::ExprType::Class(class_ptr) => {
            let class = unsafe { &*class_ptr };
            match llvm_ret {
                BasicValueEnum::PointerValue(p) => Value::Class(p, class),
                x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
            }
        },
        x => panic!("Expect Class, found {:?}", x),
    }
}

fn gen_dot_invoke(
    dot_invoke: &tree::DotInvoke,
    context: &FnContext
) -> Value {
    let func = unsafe { &*dot_invoke.invoke.func_ref.get().unwrap() };

    let mut llvm_params = vec![];
    llvm_params.push(convert(&gen_expr(&dot_invoke.expr, context)));
    for param in &dot_invoke.invoke.args {
        llvm_params.push(convert(&gen_expr(param, context)));
    }

    let llvm_ret = context.builder.build_call(func.llvm_ref.get().unwrap(), &llvm_params, &dot_invoke.invoke.name);

    match func.return_type.get().unwrap() {
        tree::ExprType::Void => Value::Void,
        tree::ExprType::Class(class_ptr) => {
            let class = unsafe { &*class_ptr };
            match llvm_ret.try_as_basic_value().left().unwrap() {
                BasicValueEnum::PointerValue(p) => Value::Class(p, class),
                x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
            }
        },
        x => panic!("Expect Class, found {:?}", x),
    }
}

fn gen_class_instance(
    class_instance: &tree::ClassInstance,
    context: &FnContext
) -> Value {
    let class = unsafe { &*class_instance.class_ref.get().unwrap() };

    let instance = native::gen_malloc(&class.llvm_struct_type_ref.get().unwrap(), context);

    for (index, param) in class_instance.params.iter().enumerate() {
        let value = gen_expr(param, context);

        let ptr = unsafe {
            context.builder.build_in_bounds_gep(
                instance,
                &[context.context.i32_type().const_int(0, false), context.context.i32_type().const_int(index as u64, false)],
                "gep")
        };

        context.builder.build_store(ptr, convert(&value));

    }

    Value::Class(instance, class_instance.class_ref.get().unwrap())
}

fn gen_llvm_class_instance(
    class_instance: &tree::LlvmClassInstance,
    context: &FnContext
) -> Value {
    let class = unsafe { &*class_instance.class_ref.get().unwrap() };

    let instance = native::gen_malloc(&class.llvm_struct_type_ref.get().unwrap(), context);

    for (index, param) in class_instance.params.iter().enumerate() {
        let value = match gen_expr(param, context) {
            Value::LlvmNumber(i) => Value::LlvmNumber(i),
            Value::LlvmBoolean(i) => Value::LlvmBoolean(i),
            Value::LlvmString(i) => Value::LlvmString(i),
            x => panic!("Expect Llvm types, found {:?}", x)
        };

        let ptr = unsafe {
            context.builder.build_struct_gep(instance, index as u32, "gep for member")
        };

        context.builder.build_store(ptr, convert(&value));
    }

    Value::Class(instance, class_instance.class_ref.get().unwrap())
}

fn gen_llvm_number(
    number: &tree::LlvmNumber,
    context: &FnContext,
) -> Value {
    Value::LlvmNumber(context.context.i32_type().const_int(number.value as u64, false))
}

fn gen_llvm_boolean(
    boolean: &tree::LlvmBoolean,
    context: &FnContext,
) -> Value {
   Value::LlvmBoolean(context.context.bool_type().const_int(boolean.value as u64, false))
}

fn gen_llvm_string(
    literal_string: &tree::LlvmString,
    context: &FnContext
) -> Value {
    let i8_type = context.context.i8_type();
    let i32_type = context.context.i32_type();

    let string = native::gen_malloc(&context.core.string_struct_type, context);

    let array_type = i8_type.array_type((literal_string.value.len() + 1) as u32);
    let array = native::gen_malloc_array(&array_type, context);

    for (index, c) in literal_string.value.chars().enumerate() {
        let p = unsafe {
            context.builder.build_in_bounds_gep(
                array,
                &[i32_type.const_int(0, false), i32_type.const_int(index as u64, false)],
                "gep")
        };
        context.builder.build_store(p, i8_type.const_int(c as u64, false));
    }
    // Store string terminating symbol
    let last = unsafe {
        context.builder.build_in_bounds_gep(
            array,
            &[i32_type.const_int(0, false), i32_type.const_int(literal_string.value.len() as u64, false)],
            "gep")
    };
    context.builder.build_store(last, i8_type.const_int(0, false));

    let size = i32_type.const_int((literal_string.value.len() + 1) as u64, false);
    let size_pointer = unsafe { context.builder.build_struct_gep(string, 0, "gep") };
    context.builder.build_store(size_pointer, size);

    let content_pointer = unsafe { context.builder.build_struct_gep(string, 1, "gep") };
    let array_pointer = unsafe {
        context.builder.build_in_bounds_gep(
            array,
            &[
                context.context.i32_type().const_int(0, false),
                context.context.i32_type().const_int(0, false)
            ],
            "array_pointer",
        )
    };
    context.builder.build_store(content_pointer, array_pointer);

    Value::LlvmString(string)
}

fn gen_if_else(
    if_else: &tree::IfElse,
    context: &FnContext,
) -> Value {
    let comparison = match gen_expr(&if_else.cond, context) {
        Value::Class(ptr, class_ptr) => {
            let class = unsafe { &*class_ptr };
            let boolean_ptr = unsafe { context.builder.build_struct_gep(ptr, 0, "gep for @Boolean") };
            let boolean_instance = match context.builder.build_load(boolean_ptr, "load @Boolean") {
                BasicValueEnum::PointerValue(p) => p,
                x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
            };
            let llvm_boolean_ptr = unsafe { context.builder.build_struct_gep(boolean_instance, 0, "gep for llvm boolean") };
            match context.builder.build_load(llvm_boolean_ptr, "load llvm boolean") {
                BasicValueEnum::IntValue(i) => i,
                x => panic!("Expect BasicValueEnum::IntValue, found {:?}", x),
            }
        },
        _ => panic!(""),
    };
    let true_block = context.context.append_basic_block(context.func, "true_block");
    let false_block = context.context.append_basic_block(context.func, "false_block");
    let end_block = context.context.append_basic_block(context.func, "end");
    let jump_instruction = context.builder.build_conditional_branch(comparison, &true_block, &false_block);

    context.builder.position_at_end(&true_block);
    let true_value = gen_expr(&if_else.true_br, context);

    context.builder.position_at_end(&false_block);
    let false_value = gen_expr(&if_else.false_br, context);

    match (&true_value, &false_value) {
        (Value::Class(_, true_class_ptr), Value::Class(_, false_class_ptr)) => {
            let class = unsafe { &**true_class_ptr };

            context.builder.position_before(&jump_instruction);
            let ret_pointer = context.builder.build_alloca(class.llvm_struct_type_ref.get().unwrap().ptr_type(AddressSpace::Generic), "ret_if_else");

            context.builder.position_at_end(&true_block);
            context.builder.build_store(ret_pointer, convert(&true_value));
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&false_block);
            context.builder.build_store(ret_pointer, convert(&false_value));
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&end_block);
            match context.builder.build_load(ret_pointer, "load_ret_if_else") {
                BasicValueEnum::PointerValue(i) => Value::Class(i, *true_class_ptr),
                x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
            }
        },
        (Value::Void, _) | (_, Value::Void) => {
            context.builder.position_at_end(&true_block);
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&false_block);
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&end_block);

            Value::Void
        },
        _ => panic!("")
    }
}

fn gen_read_var(
    var: &tree::ReadVar,
    context: &FnContext,
) -> Value {
    if let Some(param_index) = var.member_param_index.get() {
        let assignment = unsafe { &*var.assignment_ref.get().unwrap() };

        let llvm_class_instance_pointer = match context.builder.build_load(assignment.llvm_ref.get().unwrap(), "load class instance") {
            BasicValueEnum::PointerValue(p) => p,
            _ => panic!()
        };

        let ptr = unsafe { context.builder.build_struct_gep(llvm_class_instance_pointer, param_index as u32, "gep") };
        let llvm_ret = context.builder.build_load(ptr, &format!("load member {}", var.name));

        match var.tpe.get().unwrap() {
            tree::ExprType::Class(class_ptr) => {
                let class = unsafe { &*class_ptr };
                match llvm_ret {
                    BasicValueEnum::PointerValue(p) => Value::Class(p, class),
                    _ => panic!(""),
                }
            },
            tree::ExprType::Void => Value::Void,
            _ => panic!(""),
        }
    } else {
        let assignment = unsafe { &*var.assignment_ref.get().unwrap() };
        let i32_type = context.context.i32_type();
        let value = context.builder.build_load(
            assignment.llvm_ref.get().unwrap(),
            &var.name
        );
        match var.tpe.get().unwrap() {
            tree::ExprType::Class(class) => {
                let ptr = match value {
                    BasicValueEnum::PointerValue(p) => p,
                    _ => panic!()
                };
                Value::Class(ptr, class)
            },
            _ => panic!(),
        }
    }
}

fn gen_assignment(
    assignment: &tree::Assignment,
    context: &FnContext,
) -> Value {
    let expr = gen_expr(&assignment.expr, context);

   let ptr = match expr {
       Value::Class(p, class) => {
           let struct_type = unsafe { (&*class).llvm_struct_type_ref.get().unwrap() };
           let ptr_type = struct_type.ptr_type(AddressSpace::Generic);
           context.builder.build_alloca(ptr_type, &assignment.var.name)

       },
       _ => panic!("Unknown expr")
   } ;


    context.builder.build_store(ptr, convert(&expr));
    assignment.var.llvm_ref.replace(Some(ptr));
    Value::Void
}

fn gen_invoke(
    invoke: &tree::Invoke,
    context: &FnContext,
) -> Value {
    let func = unsafe { &*invoke.func_ref.get().unwrap() };

    let mut llvm_params = vec![];
    for param in &invoke.args {
        llvm_params.push(convert(&gen_expr(param, context)));
    }

    let llvm_ret = context.builder.build_call(func.llvm_ref.get().unwrap(), &llvm_params, &invoke.name);
    match func.return_type.get().unwrap() {
        tree::ExprType::Void => Value::Void,
        tree::ExprType::Class(class_ptr) => {
            let class = unsafe { &*class_ptr };
            match llvm_ret.try_as_basic_value().left().unwrap() {
                BasicValueEnum::PointerValue(p) => Value::Class(p, class),
                x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
            }
        },
        _ => panic!(""),
    }
}
