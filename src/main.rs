use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;

extern crate inkwell;

use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, BasicValue, IntValue};
use inkwell::basic_block::BasicBlock;
use inkwell::targets::{InitializationConfig, Target, TargetMachine, RelocMode, CodeModel, FileType};

mod lilit;
mod ast;
mod semantics;
mod scope;

fn gen_mod<'a>(
    module: Rc<semantics::Mod<'a>>,
    context: &Context,
    builder: &Builder,
    funcs: &HashMap<String, Rc<semantics::Func<'a>>>,
) -> Module {
    let llvm_module = context.create_module("main");
    for unit in &module.units {
        gen_mod_unit(Rc::clone(unit), &llvm_module, &context, &builder, &funcs);
    }
    return llvm_module;
}

fn gen_mod_unit<'a>(
    unit: Rc<semantics::ModUnit<'a>>,
    module: &Module,
    context: &Context,
    builder: &Builder,
    funcs: &HashMap<String, Rc<semantics::Func<'a>>>,
) {
    match *unit {
        semantics::ModUnit::Func { ref func, syntax: _ } => {
            gen_func(Rc::clone(func), &module, &context, &builder, &funcs);
        },
        _ => (),
    }
}

fn gen_func<'a>(
    func: Rc<semantics::Func<'a>>,
    module: &Module,
    context: &Context,
    builder: &Builder,
    funcs: &HashMap<String, Rc<semantics::Func<'a>>>,
) {
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);

    let function = module.add_function(&*func.syntax.id.name, fn_type, None);
    func.llvm_ref.replace(Some(function));

    for (index, expr) in func.exprs.iter().enumerate() {
        let basic_block = context.append_basic_block(&function, &format!("block_{}", index));
        if index > 0 {
            builder.build_unconditional_branch(&basic_block);
        }

        builder.position_at_end(&basic_block);

        let ret = gen_expr(Rc::clone(expr), &module, &context, &builder, &funcs);

        if index == (func.exprs.len() - 1) {
            builder.build_return(Some(&ret));
        }
    }
}

fn gen_expr<'a>(
    expr: Rc<semantics::Expr<'a>>,
    module: &Module,
    context: &Context,
    builder: &Builder,
    funcs: &HashMap<String, Rc<semantics::Func<'a>>>,
) -> IntValue {
    match *expr {
        semantics::Expr::Invoke { ref invoke, syntax: _ } => {
            gen_invoke(Rc::clone(invoke), &module, &context, &builder, &funcs)
        },
        semantics::Expr::Num { ref num, syntax: _ } => {
            gen_num(Rc::clone(num), &module, &context, &builder)
        },
        semantics::Expr::Assignment { ref assignment, syntax: _ } => {
            gen_assignment(Rc::clone(assignment), &module, &context, &builder, &funcs)
        },
        semantics::Expr::ReadVar { ref read_var, syntax: _ } => {
            gen_read_var(Rc::clone(read_var), &module, &context, &builder, &funcs)
        },
    }
}

fn gen_read_var<'a>(
    var: Rc<semantics::ReadVar>,
    module: &Module,
    context: &Context,
    builder: &Builder,
    funcs: &HashMap<String, Rc<semantics::Func<'a>>>,
) -> IntValue {
    let value = builder.build_load(var.origin.llvm_ref.borrow().unwrap(), "deref");
    value.into_int_value()
}

fn gen_assignment<'a>(
    assignment: Rc<semantics::Assignment<'a>>,
    module: &Module,
    context: &Context,
    builder: &Builder,
    funcs: &HashMap<String, Rc<semantics::Func<'a>>>,
) -> IntValue {
    let i32_type = context.i32_type();
    let ptr = builder.build_alloca(i32_type, &assignment.var.syntax.id.name);

    let expr = gen_expr(Rc::clone(&assignment.expr), &module, &context, &builder, &funcs);

    builder.build_store(ptr, expr);
    assignment.var.llvm_ref.replace(Some(ptr));
    expr
}

fn gen_num<'a>(
    num: Rc<semantics::Num>,
    module: &Module,
    context: &Context,
    builder: &Builder,
) -> IntValue {
    let i32_type = context.i32_type();
    i32_type.const_int(num.value as u64, false)
}

fn gen_invoke<'a>(
    invoke: Rc<semantics::Invoke<'a>>,
    module: &Module,
    context: &Context,
    builder: &Builder,
    funcs: &HashMap<String, Rc<semantics::Func<'a>>>,
) -> IntValue {
    let func_opt = invoke.func_opt.borrow();
    let func = func_opt.clone().unwrap();
    let llvm_ref_opt = func.llvm_ref.borrow();
    builder.build_call(llvm_ref_opt.clone().unwrap(), &[], &invoke.syntax.id.name).try_as_basic_value().left().unwrap().into_int_value()
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

fn build_invoke<'a>(invoke: Rc<ast::Invoke>) -> semantics::Invoke<'a> {
    semantics::Invoke {
        func_opt: RefCell::new(None),
        syntax: Rc::clone(&invoke),
    }
}

fn build_num(num: Rc<ast::Num>) -> semantics::Num {
    semantics::Num {
        value: (*num).value,
        syntax: Rc::clone(&num),
    }
}

fn build_id(id: Rc<ast::Id>) -> semantics::Id {
    semantics::Id {
        syntax: Rc::clone(&id)
    }
}

fn build_read_var<'a>(
    var: Rc<ast::Var>,
    scope: &mut scope::Scope<'a>,
) -> semantics::ReadVar {
    let origin = match scope.read(&var.id.name) {
        Some(scope::ScopeValue::Var(var)) => var,
        _ => panic!("The variable {} is not found", var.id.name),
    };
    semantics::ReadVar {
        origin: Rc::clone(origin),
        syntax: Rc::clone(&var),
    }

}

fn build_var(
    var: Rc<ast::Var>,
) -> semantics::Var {
    semantics::Var {
        llvm_ref: RefCell::new(None),
        id: Rc::new(build_id(Rc::clone(&var.id))),
        syntax: Rc::clone(&var)
    }
}

fn build_assignment<'a>(
    assignment: Rc<ast::Assignment>,
    scope: &mut scope::Scope<'a>,
) -> semantics::Assignment<'a> {
    let var = Rc::new(build_var(Rc::clone(&assignment.var)));
    scope.declare(assignment.var.id.name.to_string(), scope::ScopeValue::Var(Rc::clone(&var)));
    semantics::Assignment {
        var: var,
        expr: Rc::new(build_expr(Rc::clone(&assignment.expr), scope)),
        syntax: Rc::clone(&assignment),
    }
}

fn build_expr<'a>(
    expr: Rc<ast::Expr>,
    scope: &mut scope::Scope<'a>,
) -> semantics::Expr<'a> {
    match *expr {
        ast::Expr::Invoke(ref i) => semantics::Expr::Invoke {
            invoke: Rc::new(build_invoke(Rc::clone(i))),
            syntax: Rc::clone(&expr),
        },
        ast::Expr::Num(ref n) => semantics::Expr::Num {
            num: Rc::new(build_num(Rc::clone(n))),
            syntax: Rc::clone(&expr),
        },
        ast::Expr::Assignment(ref a) => semantics::Expr::Assignment {
            assignment: Rc::new(build_assignment(Rc::clone(a), scope)),
            syntax: Rc::clone(&expr),
        },
        ast::Expr::Var(ref v) => semantics::Expr::ReadVar {
            read_var: Rc::new(build_read_var(Rc::clone(v), scope)),
            syntax: Rc::clone(&expr),
        },
    }
}

fn build_func<'a>(
    func: Rc<ast::Func>,
    scope: &mut scope::Scope<'a>,
) -> semantics::Func<'a> {
    let mut vec = Vec::new();
    scope.enter();

    for expr in &(*func).exprs {
       vec.push(Rc::new(build_expr(Rc::clone(expr), scope)))
    }
    scope.leave();

    semantics::Func { llvm_ref: RefCell::new(None), exprs: vec, syntax: Rc::clone(&func) }

}

fn build_class<'a>(class: Rc<ast::Class>) -> semantics::Class<'a> {
    semantics::Class { extends: vec![], methods: vec![], syntax: Rc::clone(&class) }
}

fn build_mod_unit<'a>(
    unit: Rc<ast::ModUnit>,
    scope: &mut scope::Scope<'a>,
) -> semantics::ModUnit<'a> {
    match *unit {
      ast::ModUnit::Func(ref func) => semantics::ModUnit::Func {
          func: Rc::new(build_func(Rc::clone(func), scope)),
          syntax: Rc::clone(&unit),
      },
      ast::ModUnit::Class(ref class) => semantics::ModUnit::Class {
          class: Rc::new(build_class(Rc::clone(class))),
          syntax: Rc::clone(&unit),
      },
    }
}


fn build_mod<'a>(
    m: Rc<ast::Mod>,
    scope: &mut scope::Scope<'a>,
) -> semantics::Mod<'a> {
    let mut vec = Vec::new();
    scope.enter();

    for unit in &(*m).units {
       vec.push(Rc::new(build_mod_unit(Rc::clone(unit), scope)))
    }

    scope.leave();

    semantics::Mod { units: vec, syntax: Rc::clone(&m) }
}

fn register_funcs<'a>(root: Rc<semantics::Mod<'a>>, funcs: &mut HashMap<String, Rc<semantics::Func<'a>>>) {
    for unit in &(*root).units {
        match **unit {
            semantics::ModUnit::Func { ref func, syntax: _ } => {
                funcs.insert(func.syntax.id.name.to_string(), Rc::clone(&func));
            },
            _ => (),
        }
    }
}

fn hydrate_funcs<'a>(root: Rc<semantics::Mod<'a>>, funcs: &HashMap<String, Rc<semantics::Func<'a>>>) {
    for unit in &(*root).units {
        match **unit {
            semantics::ModUnit::Func { ref func, syntax: _ } => {
                for expr in &func.exprs {
                    match **expr {
                        semantics::Expr::Invoke { ref invoke, syntax: _ } => {
                            invoke.func_opt.replace(funcs.get(&invoke.syntax.id.name).map(|v| Rc::clone(v)));
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
        let mut scope = scope::Scope { levels: Vec::new() };
        scope.enter();
        let mut root = Rc::new(build_mod(Rc::clone(_ok_tree), &mut scope));
        scope.leave();
        println!("{:?}\n", root);

        let mut funcs = HashMap::new();
        register_funcs(Rc::clone(&root), &mut funcs);
        println!("{:?}\n", funcs);

        hydrate_funcs(Rc::clone(&root), &funcs);

        println!("{:?}\n", root);

        Target::initialize_native(&InitializationConfig::default()).unwrap();

        let context = Context::create();
        let builder = context.create_builder();
        let module = gen_mod(Rc::clone(&root), &context, &builder, &funcs);

        let triple = TargetMachine::get_default_triple().to_string();
        let target = Target::from_triple(&triple).unwrap();
        let target_machine = target.create_target_machine(&triple, "generic", "", OptimizationLevel::Default, RelocMode::Default, CodeModel::Default).unwrap();

        let path =  Path::new("./output.o");
        let result = target_machine.write_to_file(&module, FileType::Object, &path);
        println!("---- LLVM IR ----");
        module.print_to_stderr();

    //     // This is an object file. In order to run it as a binary,
    //     // we need to link it using `cc output.o -o output`.
    //     // Now you can run `./output`.
    }
}

