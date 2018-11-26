extern crate inkwell;

use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, BasicValue, IntValue};
use inkwell::basic_block::BasicBlock;
use inkwell::targets::{InitializationConfig, Target, TargetMachine, RelocMode, CodeModel, FileType};

use semantics::tree;

pub fn generate(
    module: &tree::Mod,
    context: &Context,
    builder: &Builder,
) -> Module {
    let llvm_module = context.create_module("main");
    for unit in &module.units {
        gen_mod_unit(unit, &llvm_module, &context, &builder);
    }
    return llvm_module;
}

fn gen_mod_unit(
    unit: &tree::ModUnit,
    module: &Module,
    context: &Context,
    builder: &Builder,
) {
    match unit {
        tree::ModUnit::Func { ref func } => {
            gen_func(func, &module, &context, &builder);
        },
        _ => (),
    }
}

fn gen_func(
    func: &tree::Func,
    module: &Module,
    context: &Context,
    builder: &Builder,
) {
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);

    let function = module.add_function(&func.name, fn_type, None);
    func.llvm_ref.replace(Some(function));

    for (index, expr) in func.exprs.iter().enumerate() {
        let basic_block = context.append_basic_block(&function, &format!("block_{}", index));
        if index > 0 {
            builder.build_unconditional_branch(&basic_block);
        }

        builder.position_at_end(&basic_block);

        let ret = gen_expr(expr, &module, &context, &builder);

        if index == (func.exprs.len() - 1) {
            builder.build_return(Some(&ret));
        }
    }
}

fn gen_expr(
    expr: &tree::Expr,
    module: &Module,
    context: &Context,
    builder: &Builder,
) -> IntValue {
    match expr {
        tree::Expr::Invoke { ref invoke } => {
            gen_invoke(invoke, &module, &context, &builder)
        },
        tree::Expr::Num { ref num } => {
            gen_num(num, &module, &context, &builder)
        },
        tree::Expr::Assignment { ref assignment } => {
            gen_assignment(assignment, &module, &context, &builder)
        },
        tree::Expr::ReadVar { ref read_var } => {
            gen_read_var(read_var, &module, &context, &builder)
        },
        tree::Expr::LiteralString { ref literal_string } => {
//            gen_read_var(read_var, &module, &context, &builder)
            panic!("")
        },
    }
}

fn gen_read_var(
    var: &tree::ReadVar,
    module: &Module,
    context: &Context,
    builder: &Builder,
) -> IntValue {
    let assignment = unsafe { &*var.assignment_ref.get().unwrap() };
    let value = builder.build_load(assignment.llvm_ref.get().unwrap(), "deref");
    value.into_int_value()
}

fn gen_assignment(
    assignment: &tree::Assignment,
    module: &Module,
    context: &Context,
    builder: &Builder,
) -> IntValue {
    let i32_type = context.i32_type();
    let ptr = builder.build_alloca(i32_type, &assignment.var.name);

    let expr = gen_expr(&assignment.expr, &module, &context, &builder);

    builder.build_store(ptr, expr);
    assignment.var.llvm_ref.replace(Some(ptr));
    expr
}

fn gen_num(
    num: &tree::Num,
    module: &Module,
    context: &Context,
    builder: &Builder,
) -> IntValue {
    let i32_type = context.i32_type();
    i32_type.const_int(num.value as u64, false)
}

fn gen_invoke(
    invoke: &tree::Invoke,
    module: &Module,
    context: &Context,
    builder: &Builder,
) -> IntValue {
    let func = unsafe { &*invoke.func_ref.get().unwrap() };
    builder.build_call(func.llvm_ref.get().unwrap(), &[], &invoke.name).try_as_basic_value().left().unwrap().into_int_value()
}
