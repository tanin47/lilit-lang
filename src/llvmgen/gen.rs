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

#[derive(Debug)]
pub enum Value {
    Void,
    Number(IntValue),
    Boolean(IntValue),
    String(PointerValue),
    Class(PointerValue, *const tree::Class),
}

pub fn convert(value: &Value) -> BasicValueEnum {
    match value {
        Value::Number(i) => (*i).into(),
        Value::Boolean(b) => (*b).into(),
        Value::String(p) => (*p).into(),
        Value::Class(p, c) => (*p).into(),
        Value::Void => panic!("can't convert void"),
    }
}

pub struct Core {
    pub string_struct_type: StructType
}

struct ModContext<'a, 'b, 'c, 'd> {
    module: &'a Module,
    context: &'b Context,
    builder: &'c Builder,
    core: &'d Core,
}

pub struct FnContext<'a, 'b, 'c, 'd, 'e> {
    pub func: &'a FunctionValue,
    pub module: &'b Module,
    pub context: &'c Context,
    pub builder: &'d Builder,
    pub core: &'e Core,
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
    };
    {
        let context = ModContext {
            module: &llvm_module,
            context,
            builder,
            core: &core
        };
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
        tree::ModUnit::Class(ref class) => gen_class(class, context),
    }
}

fn gen_class(
    class: &tree::Class,
    context: &ModContext
) {
    let mut type_enums: Vec<BasicTypeEnum> = vec![];
    for param in &class.params {
        type_enums.push(match param.tpe.get().unwrap() {
            tree::ExprType::Number => context.context.i32_type().into(),
            tree::ExprType::String => context.core.string_struct_type.ptr_type(AddressSpace::Generic).into(),
            tree::ExprType::Boolean => context.context.i32_type().into(),
            // TODO: we should support a class type here. This can have circular dependency
            // This actually might need opaque type of something
            _ => panic!()
        });
    }
    let class_struct = StructType::struct_type(
        &type_enums,
        false
    );
    class.llvm_struct_type_ref.set(Some(class_struct));

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
            tree::ExprType::Number => context.context.i32_type().into(),
            tree::ExprType::String => context.core.string_struct_type.ptr_type(AddressSpace::Generic).into(),
            tree::ExprType::Boolean => context.context.bool_type().into(),
            tree::ExprType::Class(class_ptr) => {
                let class = unsafe { &*class_ptr };
                class.llvm_struct_type_ref.get().unwrap().ptr_type(AddressSpace::Generic).into()
            },
            x => panic!("Unrecognized param: {:?}", x)
        });
    }

    let fn_type = match method.return_type.get().unwrap() {
        tree::ExprType::Void => context.context.void_type().fn_type(&param_types, false),
        tree::ExprType::Number => context.context.i32_type().fn_type(&param_types, false),
        tree::ExprType::String => context.core.string_struct_type.ptr_type(AddressSpace::Generic).fn_type(&param_types, false),
        tree::ExprType::Boolean => context.context.bool_type().fn_type(&param_types, false),
        tree::ExprType::Class(class_ptr) => {
            let class = unsafe { &*class_ptr };
            class.llvm_struct_type_ref.get().unwrap().ptr_type(AddressSpace::Generic).fn_type(&param_types, false)
        },
    };

    let func_name = format!("__{}__{}", class.name, method.name);
    let function = context.module.add_function(&func_name, fn_type, None);
    method.llvm_ref.replace(Some(function));

    let first_block = context.context.append_basic_block(&function, "first_block");
    context.builder.position_at_end(&first_block);

    for (index, param) in method.params.iter().enumerate() {
        let tpe: BasicTypeEnum = match param.tpe.get().unwrap() {
            tree::ExprType::Number => context.context.i32_type().into(),
            tree::ExprType::String => context.core.string_struct_type.ptr_type(AddressSpace::Generic).into(),
            tree::ExprType::Boolean => context.context.i32_type().into(),
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
        if class.name == "@I8" {
            if method.name == "add" {
                let first = native::int8::get_llvm_value_from_var(&method.params[0].var, &fn_context);
                let second = native::int8::get_llvm_value_from_var(&method.params[1].var, &fn_context);
                let sum = context.builder.build_int_nsw_add(first, second, "@I8.add");
                let i8_value = native::int8::instantiate_from_value(sum.into(), &class, &fn_context);
                context.builder.build_return(Some(&convert(&i8_value)));
            } else if method.name == "to_num" {
                let first = native::int8::get_llvm_value_from_var(&method.params[0].var, &fn_context);
                context.builder.build_return(Some(&first));
            } else {
                panic!("Unrecognized LLVM method: {}.{}", class.name, method.name);
            }
        } else {
            panic!("Unrecognized LLVM class: {}", class.name);
        }
    } else {
        for (index, expr) in method.exprs.iter().enumerate() {
            let ret = gen_expr(expr, &fn_context);
            if index == (method.exprs.len() - 1) {
                context.builder.build_return(Some(&convert(&ret)));
            }
        }
    }
}

fn gen_func(
    func: &tree::Func,
    context: &ModContext,
) {
    let mut param_types: Vec<BasicTypeEnum> = vec![];
    for param in &func.params {
       param_types.push(match param.tpe.get().unwrap() {
           tree::ExprType::Number => context.context.i32_type().into(),
           tree::ExprType::String => context.core.string_struct_type.ptr_type(AddressSpace::Generic).into(),
           tree::ExprType::Boolean => context.context.bool_type().into(),
           tree::ExprType::Class(class_ptr) => {
               let class = unsafe { &*class_ptr };
               class.llvm_struct_type_ref.get().unwrap().ptr_type(AddressSpace::Generic).into()
           },
           x => panic!("Unrecognized param: {:?}", x)
       });
    }

    let fn_type = match func.return_type.get().unwrap() {
        tree::ExprType::Void => context.context.void_type().fn_type(&param_types, false),
        tree::ExprType::Number => context.context.i32_type().fn_type(&param_types, false),
        tree::ExprType::String => context.core.string_struct_type.ptr_type(AddressSpace::Generic).fn_type(&param_types, false),
        tree::ExprType::Boolean => context.context.bool_type().fn_type(&param_types, false),
        tree::ExprType::Class(class_ptr) => {
            let class = unsafe { &*class_ptr };
            class.llvm_struct_type_ref.get().unwrap().ptr_type(AddressSpace::Generic).fn_type(&param_types, false)
        },
    };

    let function = context.module.add_function(&func.name, fn_type, None);
    func.llvm_ref.replace(Some(function));

    let first_block = context.context.append_basic_block(&function, "first_block");
    context.builder.position_at_end(&first_block);

    let fn_context = FnContext {
        func: &function,
        module: context.module,
        context: context.context,
        builder: context.builder,
        core: context.core,
    };

    for (index, expr) in func.exprs.iter().enumerate() {
        let ret = gen_expr(expr, &fn_context);
        if index == (func.exprs.len() - 1) {
            context.builder.build_return(Some(&convert(&ret)));
        }
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
        tree::Expr::Num(ref num) => gen_num(num, context),
        tree::Expr::Assignment(ref assignment) => gen_assignment(assignment, context),
        tree::Expr::ReadVar(ref read_var) => gen_read_var(read_var, context),
        tree::Expr::LiteralString(ref literal_string) => gen_string(literal_string, context),
        tree::Expr::Boolean(ref boolean) => gen_boolean(boolean, context),
        tree::Expr::Comparison(ref comparison) => gen_comparison(comparison, context),
        tree::Expr::IfElse(ref if_else) => gen_if_else(if_else, context),
        tree::Expr::ClassInstance(ref class_instance) => gen_class_instance(class_instance, context),
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
        context.builder.build_in_bounds_gep(
            llvm_expr,
            &[context.context.i32_type().const_int(0, false), context.context.i32_type().const_int((dot_member.member.param_index.get().unwrap()) as u64, false)],
            "gep")
    };

    let llvm_ret = context.builder.build_load(ptr, &format!("load param {}", dot_member.member.name));

    match dot_member.tpe.get().unwrap() {
        tree::ExprType::Number => {
            match llvm_ret {
                BasicValueEnum::IntValue(i) => Value::Number(i),
                _ => panic!(""),
            }
        },
        tree::ExprType::String => {
            match llvm_ret {
                BasicValueEnum::PointerValue(p) => Value::String(p),
                _ => panic!(""),
            }
        },
        tree::ExprType::Void => Value::Void,
        _ => panic!(""),
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
        tree::ExprType::Number => {
            match llvm_ret.try_as_basic_value().left().unwrap() {
                BasicValueEnum::IntValue(i) => Value::Number(i),
                x => panic!("Expect BasicValueEnum::IntValue, found {:?}", x),
            }
        },
        tree::ExprType::String => {
            match llvm_ret.try_as_basic_value().left().unwrap() {
                BasicValueEnum::PointerValue(p) => Value::String(p),
                x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
            }
        },
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

fn gen_class_instance(
    class_instance: &tree::ClassInstance,
    context: &FnContext
) -> Value {
    let class = unsafe { &*class_instance.class_ref.get().unwrap() };

    let instance= context.builder.build_alloca(class.llvm_struct_type_ref.get().unwrap(), "class");

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

fn gen_boolean(
    boolean: &tree::Boolean,
    context: &FnContext,
) -> Value {
   Value::Boolean(context.context.bool_type().const_int(boolean.value as u64, false))
}

fn gen_comparison(
    comparison: &tree::Comparison,
    context: &FnContext,
) -> Value {
    let var = match gen_read_var(&comparison.left, context) {
        Value::Number(i) => i,
        _ => panic!("Unable to read var into IntValue")
    };
    let num = match gen_num(&comparison.right, context) {
        Value::Number(i) => i,
        _ => panic!("")
    };
    Value::Boolean(context.builder.build_int_compare(IntPredicate::SGT, var, num, "cond"))
}

fn gen_if_else(
    if_else: &tree::IfElse,
    context: &FnContext,
) -> Value {
    let comparison = match gen_comparison(&if_else.cond, context) {
        Value::Boolean(i) => i,
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
        (Value::Number(_), Value::Number(_)) => {
            context.builder.position_before(&jump_instruction);
            let ret_pointer = context.builder.build_alloca(context.context.i32_type(), "ret_if_else");

            context.builder.position_at_end(&true_block);
            context.builder.build_store(ret_pointer, convert(&true_value));
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&false_block);
            context.builder.build_store(ret_pointer, convert(&false_value));
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&end_block);
            match context.builder.build_load(ret_pointer, "load_ret_if_else") {
                BasicValueEnum::IntValue(i) => Value::Number(i),
                _ => panic!("")
            }
        },
        (Value::String(_), Value::String(_)) => {
            context.builder.position_before(&jump_instruction);
            let ret_pointer = context.builder.build_alloca(context.core.string_struct_type.ptr_type(AddressSpace::Generic), "ret_if_else");

            context.builder.position_at_end(&true_block);
            context.builder.build_store(ret_pointer, convert(&true_value));
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&false_block);
            context.builder.build_store(ret_pointer, convert(&false_value));
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&end_block);
            match context.builder.build_load(ret_pointer, "load_ret_if_else") {
                BasicValueEnum::PointerValue(i) => Value::String(i),
                _ => panic!("")
            }
        },
        (Value::Boolean(_), Value::Boolean(_)) => {
            context.builder.position_before(&jump_instruction);
            let ret_pointer = context.builder.build_alloca(context.context.i32_type(), "ret_if_else");

            context.builder.position_at_end(&true_block);
            context.builder.build_store(ret_pointer, convert(&true_value));
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&false_block);
            context.builder.build_store(ret_pointer, convert(&false_value));
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&end_block);
            match context.builder.build_load(ret_pointer, "load_ret_if_else") {
                BasicValueEnum::IntValue(i) => Value::Boolean(i),
                _ => panic!("")
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

fn gen_string(
    literal_string: &tree::LiteralString,
    context: &FnContext
) -> Value {
    let i8_type = context.context.i8_type();
    let i32_type = context.context.i32_type();

    let string = context.builder.build_alloca(context.core.string_struct_type, "string");

    let array_type = i8_type.array_type((literal_string.content.len() + 1) as u32);
    let array = context.builder.build_alloca(array_type, "string_array");

    for (index, c) in literal_string.content.chars().enumerate() {
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
            &[i32_type.const_int(0, false), i32_type.const_int(literal_string.content.len() as u64, false)],
            "gep")
    };
    context.builder.build_store(last, i8_type.const_int(0, false));

    let size = i32_type.const_int((literal_string.content.len() + 1) as u64, false);

    let size_pointer = unsafe {
        context.builder.build_in_bounds_gep(
            string,
            &[i32_type.const_int(0, false), i32_type.const_int(0, false)],
            "gep"
        )
    };
    context.builder.build_store(size_pointer, size);

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

        let ptr = unsafe {
            context.builder.build_in_bounds_gep(
                llvm_class_instance_pointer,
                &[context.context.i32_type().const_int(0, false), context.context.i32_type().const_int((param_index as u64), false)],
                "gep")
        };

        let llvm_ret = context.builder.build_load(ptr, &format!("load member {}", var.name));

        match var.tpe.get().unwrap() {
            tree::ExprType::Number => {
                match llvm_ret {
                    BasicValueEnum::IntValue(i) => Value::Number(i),
                    _ => panic!(""),
                }
            },
            tree::ExprType::String => {
                match llvm_ret {
                    BasicValueEnum::PointerValue(p) => Value::String(p),
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
            tree::ExprType::Number => {
                match value {
                    BasicValueEnum::IntValue(i) => Value::Number(i),
                    _ => panic!()
                }
            },
            tree::ExprType::String => {
                match value {
                    BasicValueEnum::PointerValue(p) => Value::String(p),
                    _ => panic!()
                }
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
       Value::Number(_) => {
           let i32_type = context.context.i32_type();
           context.builder.build_alloca(i32_type, &assignment.var.name)
       },
       Value::String(p) => {
           let ptr_type = context.core.string_struct_type.ptr_type(AddressSpace::Generic);
           context.builder.build_alloca(ptr_type, &assignment.var.name)
       },
       Value::Class(p, class) => {
           let struct_type = unsafe { (&*class).llvm_struct_type_ref.get().unwrap() };
           let ptr_type = struct_type.ptr_type(AddressSpace::Generic);
           context.builder.build_alloca(ptr_type, &assignment.var.name)

       },
       _ => panic!("Unknow expr")
   } ;


    context.builder.build_store(ptr, convert(&expr));
    assignment.var.llvm_ref.replace(Some(ptr));
    Value::Void
}

fn gen_num(
    num: &tree::Num,
    context: &FnContext,
) -> Value {
    let i32_type = context.context.i32_type();
    Value::Number(i32_type.const_int(num.value as u64, false))
}

fn gen_invoke(
    invoke: &tree::Invoke,
    context: &FnContext,
) -> Value {
    let func = unsafe { &*invoke.func_ref.get().unwrap() };
    let llvm_ret = context.builder.build_call(func.llvm_ref.get().unwrap(), &[], &invoke.name);

    match func.return_type.get().unwrap() {
        tree::ExprType::Number => {
            match llvm_ret.try_as_basic_value().left().unwrap() {
                BasicValueEnum::IntValue(i) => Value::Number(i),
                _ => panic!(""),
            }
        },
        tree::ExprType::String => {
            match llvm_ret.try_as_basic_value().left().unwrap() {
                BasicValueEnum::PointerValue(p) => Value::String(p),
                _ => panic!(""),
            }
        },
        tree::ExprType::Void => Value::Void,
        _ => panic!(""),
    }
}
