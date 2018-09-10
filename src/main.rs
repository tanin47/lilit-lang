use std::cell::Cell;
use std::env;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::collections::HashMap;

extern crate inkwell;

use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, Symbol};
use inkwell::module::Module;
use inkwell::values::{FunctionValue, BasicValue, IntValue};
use inkwell::basic_block::BasicBlock;
use inkwell::targets::{InitializationConfig, Target, TargetMachine, RelocMode, CodeModel, FileType};

mod lilit;
mod ast;
mod semantics;

fn gen_mod<'a>(
    module: &semantics::Mod<'a>,
    context: &Context,
    builder: &Builder,
    funcs: &HashMap<String, &'a semantics::Func<'a>>,
) -> Module {
    let llvm_module = context.create_module("main");
    for unit in &module.units {
        gen_mod_unit(&unit, &llvm_module, &context, &builder, &funcs);
    }
    return llvm_module;
}

fn gen_mod_unit<'a>(
    unit: &semantics::ModUnit<'a>,
    module: &Module,
    context: &Context,
    builder: &Builder,
    funcs: &HashMap<String, &'a semantics::Func<'a>>,
) {
    match unit {
        semantics::ModUnit::Func { func, syntax: _ } => {
            gen_func(&func, &module, &context, &builder, &funcs);
        },
        _ => (),
    }
}

fn gen_func<'a>(
    func: &semantics::Func<'a>,
    module: &Module,
    context: &Context,
    builder: &Builder,
    funcs: &HashMap<String, &'a semantics::Func<'a>>,
) {
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);

    let function = module.add_function(&*func.syntax.name, &fn_type, None);
    func.llvm_ref.set(Some(function));

    for (index, expr) in func.exprs.iter().enumerate() {
        let basic_block = context.append_basic_block(&function, &format!("block_{}", index));
        if index > 0 {
            builder.build_unconditional_branch(&basic_block);
        }

        builder.position_at_end(&basic_block);

        let ret = gen_expr(&expr, &module, &context, &builder, &funcs);

        if index == (func.exprs.len() - 1) {
            builder.build_return(Some(&ret));
        }
    }

}

fn gen_expr<'a>(
    expr: &semantics::Expr<'a>,
    module: &Module,
    context: &Context,
    builder: &Builder,
    funcs: &HashMap<String, &'a semantics::Func<'a>>,
) -> IntValue {
    match expr {
        semantics::Expr::Invoke { invoke, syntax: _ } => {
            gen_invoke(&invoke, &module, &context, &builder, &funcs)
        },
        semantics::Expr::Num { num, syntax: _ } => {
            gen_num(&num, &module, &context, &builder)
        },
    }
}

fn gen_num(
    num: &semantics::Num,
    module: &Module,
    context: &Context,
    builder: &Builder,
) -> IntValue {
    let i32_type = context.i32_type();
    i32_type.const_int(num.value as u64, false)
}

fn gen_invoke<'a>(
    invoke: &semantics::Invoke<'a>,
    module: &Module,
    context: &Context,
    builder: &Builder,
    funcs: &HashMap<String, &'a semantics::Func<'a>>,
) -> IntValue {
    builder.build_call(&invoke.func_opt.get().unwrap().llvm_ref.get().unwrap(), &[], &invoke.syntax.name, false).left().unwrap().into_int_value()
}


// fn build_next_mod(
//     next_module_opt: &Option<Box<ast::Mod>>,
//     module: &Module,
//     context: &Context,
//     builder: &Builder
// ) {
//     if let Some(ref next_module) = next_module_opt {
//         add_func(&*(*next_module).func, &module, &context, &builder);
//         build_next_mod(&next_module.next_opt, &module, &context, &builder);
//     }
// }

// fn add_func(
//     func: &ast::Func,
//     module: &Module,
//     context: &Context,
//     builder: &Builder
// ) {
//     let i32_type = context.i32_type();
//     let fn_type = i32_type.fn_type(&[], false);

//     let function = module.add_function(&*func.name, &fn_type, None);
//     let basic_block = context.append_basic_block(&function, "entry");

//     builder.position_at_end(&basic_block);
//     let ret = i32_type.const_int((*func.expr).value as u64, false);
//     builder.build_return(Some(&ret));
// }

fn build_invoke<'a>(invoke: &'a ast::Invoke) -> semantics::Invoke<'a> {
    semantics::Invoke {
        func_opt: Cell::new(None),
        syntax: &invoke,
    }
}

fn build_num<'a>(num: &'a ast::Num) -> semantics::Num<'a> {
    semantics::Num {
        value: (*num).value,
        syntax: &num,
    }
}

fn build_expr<'a>(expr: &'a ast::Expr) -> semantics::Expr<'a> {
    match expr {
        ast::Expr::Invoke(i) => semantics::Expr::Invoke {
            invoke: Box::new(build_invoke(&i)),
            syntax: &expr,
        },
        ast::Expr::Num(n) => semantics::Expr::Num {
            num: Box::new(build_num(&n)),
            syntax: &expr,
        },
    }
}

fn build_func<'a>(func: &'a ast::Func) -> semantics::Func<'a> {
    let mut vec = Vec::new();

    for expr in &(*func).exprs {
       vec.push(build_expr(&expr))
    }

    semantics::Func { llvm_ref: Cell::new(None), exprs: vec, syntax: &func }
}

fn build_class<'a>(class: &'a ast::Class) -> semantics::Class<'a> {
    semantics::Class { extends: vec![], methods: vec![], syntax: &class }
}

fn build_mod_unit<'a>(unit: &'a ast::ModUnit) -> semantics::ModUnit<'a> {
  match unit {
    ast::ModUnit::Func(func) => semantics::ModUnit::Func {
        func: Box::new(build_func(&func)),
        syntax: &unit,
    },
    ast::ModUnit::Class(class) => semantics::ModUnit::Class {
        class: Box::new(build_class(&class)),
        syntax: &unit,
    },
  }
}


fn build_mod<'a>(m: &'a ast::Mod) -> semantics::Mod<'a> {
    let mut vec = Vec::new();

    for unit in &(*m).units {
       vec.push(build_mod_unit(&unit))
    }

    semantics::Mod { units: vec, syntax: &m }
}

fn register_funcs<'a>(root: &'a semantics::Mod<'a>, funcs: &mut HashMap<String, &'a semantics::Func<'a>>) {
    for unit in &(*root).units {
        match unit {
            semantics::ModUnit::Func { func, syntax: _ } => {
                funcs.insert(func.syntax.name.to_string(), func);
            },
            _ => (),
        }
    }
}

fn hydrate_funcs<'a>(root: &'a semantics::Mod<'a>, funcs: &HashMap<String, &'a semantics::Func<'a>>) {
    for unit in &(*root).units {
        match unit {
            semantics::ModUnit::Func { func, syntax: _ } => {
                for expr in &func.exprs {
                    match expr {
                        semantics::Expr::Invoke { invoke, syntax: _ } => {
                            invoke.func_opt.set(funcs.get(&invoke.syntax.name).map(|v| *v));
                        },
                        _ => (),
                    }
                }
            },
            _ => (),
        }
    }
}

fn main() {
    println!("Lilit 0.0.1\n");
    let args: Vec<String> = env::args().collect();
    let mut file = File::open(&args[1]).expect("file not found");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    let tree = lilit::ModParser::new().parse(&contents);

    println!("---- Code ----");
    println!("{}\n", contents);


    println!("{:?}\n", tree);

    // The first pass makes a hashtable for function and class.

    if let Ok(ref _ok_tree) = tree {
        let mut root = build_mod(_ok_tree);
        println!("{:?}\n", root);

        let mut funcs = HashMap::new();
        register_funcs(&root, &mut funcs);
        println!("{:?}\n", funcs);

        hydrate_funcs(&root, &funcs);

        println!("{:?}\n", root);

        Target::initialize_native(&InitializationConfig::default()).unwrap();

        let context = Context::create();
        let builder = context.create_builder();
        // let module = context.create_module("main");
        // let i32_type = context.i32_type();
        // let fn_type = i32_type.fn_type(&[], false);

        // let function = module.add_function("main", &fn_type, None);


        let module = gen_mod(&root, &context, &builder, &funcs);

        let triple = TargetMachine::get_default_triple().to_string();
        let target = Target::from_triple(&triple).unwrap();
        let target_machine = target.create_target_machine(&triple, "generic", "", OptimizationLevel::Default, RelocMode::Default, CodeModel::Default).unwrap();

        let path =  Path::new("./output.o\0");
        let result = target_machine.write_to_file(&module, FileType::Object, &path);
        println!("---- LLVM IR ----");
        module.print_to_stderr();

    //     // This is an object file. In order to run it as a binary,
    //     // we need to link it using `cc output.o -o output`.
    //     // Now you can run `./output`.
    }
}

