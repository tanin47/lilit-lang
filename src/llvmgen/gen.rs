use inkwell::AddressSpace;
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, BasicValue, IntValue};
use inkwell::basic_block::BasicBlock;
use inkwell::targets::{InitializationConfig, Target, TargetMachine, RelocMode, CodeModel, FileType};

use semantics::tree;
use inkwell::values::BasicValueEnum;
use inkwell::types::BasicTypeEnum;
use inkwell::module::Linkage;
use inkwell::values::VectorValue;
use inkwell::types::StructType;
use inkwell::types::ArrayType;
use inkwell::values::PointerValue;
use inkwell::values::StructValue;

struct Core {
   string_struct_type: StructType
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

    for unit in &module.units {
        gen_mod_unit(unit, &llvm_module, context, builder, &core);
    }
    return llvm_module;
}

fn gen_mod_unit(
    unit: &tree::ModUnit,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
) {
    match unit {
        tree::ModUnit::Func { ref func } => {
            gen_func(func, module, context, builder, core);
        },
        _ => (),
    }
}

fn gen_func(
    func: &tree::Func,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
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

        let ret = gen_expr(expr, module, context, builder, core);

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
    core: &Core,
) -> BasicValueEnum {
    match expr {
        tree::Expr::Invoke { ref invoke } => {
            gen_invoke(invoke, module, context, builder, core).into()
        },
        tree::Expr::Num { ref num } => {
            gen_num(num, module, context, builder, core).into()
        },
        tree::Expr::Assignment { ref assignment } => {
            gen_assignment(assignment, module, context, builder, core).into()
        },
        tree::Expr::ReadVar { ref read_var } => {
            gen_read_var(read_var, module, context, builder, core).into()
        },
        tree::Expr::LiteralString { ref literal_string } => {
            gen_string(literal_string, module, context, builder, core).into()
        },
    }
}

fn gen_string(
    literal_string: &tree::LiteralString,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
) -> PointerValue {
    let i8_type = context.i8_type();
    let i32_type = context.i32_type();

    let string = builder.build_alloca(core.string_struct_type, "string");

    let array_type = i8_type.array_type((literal_string.content.len() + 1) as u32);
    let array = builder.build_alloca(array_type, "string_array");

    for (index, c) in literal_string.content.chars().enumerate() {
        let p = unsafe {
            builder.build_in_bounds_gep(
                array,
                &[i32_type.const_int(0, false), i32_type.const_int(index as u64, false)],
                "gep")
        };
        builder.build_store(p, i8_type.const_int(c as u64, false));
    }
    // Store string terminating symbol
    let last = unsafe {
        builder.build_in_bounds_gep(
            array,
            &[i32_type.const_int(0, false), i32_type.const_int(literal_string.content.len() as u64, false)],
            "gep")
    };
    builder.build_store(last, i8_type.const_int(0, false));

    let size = i32_type.const_int((literal_string.content.len() + 1) as u64, false);

    let size_pointer = unsafe {
        builder.build_in_bounds_gep(
            string,
            &[i32_type.const_int(0, false), i32_type.const_int(0, false)],
            "gep"
        )
    };
    builder.build_store(size_pointer, size);

    let content_pointer = unsafe {
        builder.build_in_bounds_gep(
            string,
            &[i32_type.const_int(0, false), i32_type.const_int(1, false)],
            "gep"
        )
    };
    builder.build_store(content_pointer, array);

    println!("string {:?}", string);
    string
}

fn gen_read_var(
    var: &tree::ReadVar,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
) -> BasicValueEnum {
    let assignment = unsafe { &*var.assignment_ref.get().unwrap() };
    println!("assignment ref {:?}", assignment);
    println!("assignment llvm ref {:?}", assignment.llvm_ref);
    let i32_type = context.i32_type();
    let value = builder.build_load(
        unsafe { builder.build_in_bounds_gep(assignment.llvm_ref.get().unwrap(), &[i32_type.const_int(0, false)], "deref") },
        &var.name
    );
    println!("{:?}", value);
    value.into()
}

fn gen_assignment(
    assignment: &tree::Assignment,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
) -> BasicValueEnum {
    let expr = gen_expr(&assignment.expr, module, context, builder, core);

   let ptr = match expr {
       BasicValueEnum::IntValue(_) => {
           let i32_type = context.i32_type();
           builder.build_alloca(i32_type, &assignment.var.name)
       },
       BasicValueEnum::PointerValue(p) => {
           let ptr_type = context.i8_type().ptr_type(AddressSpace::Generic);
           builder.build_alloca(ptr_type, &assignment.var.name)
       },
       _ => panic!("Unknow expr")
   } ;


    println!("{:?}\nassignto\n{:?}", ptr, expr);
    builder.build_store(ptr, expr);
    assignment.var.llvm_ref.replace(Some(ptr));
    expr.into()
}

fn gen_num(
    num: &tree::Num,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
) -> IntValue {
    let i32_type = context.i32_type();
    i32_type.const_int(num.value as u64, false)
}

fn gen_invoke(
    invoke: &tree::Invoke,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
) -> BasicValueEnum {
    if invoke.name == "print" {
        let printf = match module.get_function("printf") {
            Some(f) => f,
            None => {
                let str_type = context.i8_type().ptr_type(AddressSpace::Generic);
                let i32_type = context.i32_type();
                let printf_type = i32_type.fn_type(&[str_type.into()], true);
                module.add_function("printf", printf_type, Some(Linkage::External))
            },
        };

        let i32_type = context.i32_type();
        let ptr_type = context.i32_type().ptr_type(AddressSpace::Generic);
        let arg = gen_expr(&invoke.arg, module, context, builder, core);
        println!("arg {:?}", arg);
        let ss = match arg {
            BasicValueEnum::PointerValue(ptr) => ptr,
            _ => panic!("fail arg"),
        };
        println!("ptr {:?}", ss);
        let s = unsafe {
            builder.build_in_bounds_gep(ss, &[i32_type.const_int(0, false), i32_type.const_int(1, false)], "")
        };
        println!("s {:?}", s);
        let l = builder.build_load(s, "load");
        println!("l {:?}", l);
        builder.build_call(printf, &[l], "").try_as_basic_value().left().unwrap().into()
    } else {
        let func = unsafe { &*invoke.func_ref.get().unwrap() };
        builder.build_call(func.llvm_ref.get().unwrap(), &[], &invoke.name).try_as_basic_value().left().unwrap().into()
    }
}
