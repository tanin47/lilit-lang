use std::env;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

extern crate inkwell;

use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, Symbol};
use inkwell::module::Module;
use inkwell::values::FunctionValue;
use inkwell::basic_block::BasicBlock;
use inkwell::targets::{InitializationConfig, Target, TargetMachine, RelocMode, CodeModel, FileType};

mod lilit;
mod ast;

fn build_mod(
    module: &ast::Mod,
    context: &Context,
    builder: &Builder
) -> Module {
    let llvm_module = context.create_module("main");
    add_func(&module.func, &llvm_module, &context, &builder);
    build_next_mod(&module.next_opt, &llvm_module, &context, &builder);
    return llvm_module;
}

fn build_next_mod(
    next_module_opt: &Option<Box<ast::Mod>>,
    module: &Module,
    context: &Context,
    builder: &Builder
) {
    if let Some(ref next_module) = next_module_opt {
        add_func(&*(*next_module).func, &module, &context, &builder);
        build_next_mod(&next_module.next_opt, &module, &context, &builder);
    }
}

fn add_func(
    func: &ast::Func,
    module: &Module,
    context: &Context,
    builder: &Builder
) {
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);

    let function = module.add_function(&*func.name, &fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);
    let ret = i32_type.const_int((*func.expr).value as u64, false);
    builder.build_return(Some(&ret));
}


fn main() {
    println!("Lilit 0.0.1");
    let args: Vec<String> = env::args().collect();
    let mut file = File::open(&args[1]).expect("file not found");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    let tree = lilit::ModParser::new().parse(&contents);
    println!("{:?}", tree);


    println!("With text:\n{}", contents);

    Target::initialize_native(&InitializationConfig::default()).unwrap();

    if let Ok(ref _ok_tree) = tree {
        let context = Context::create();
        let builder = context.create_builder();
        let module = context.create_module("main");
        let i32_type = context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);

        let function = module.add_function("main", &fn_type, None);


        let module = build_mod(_ok_tree, &context, &builder);

        let triple = TargetMachine::get_default_triple().to_string();
        let target = Target::from_triple(&triple).unwrap();
        let target_machine = target.create_target_machine(&triple, "generic", "", OptimizationLevel::Default, RelocMode::Default, CodeModel::Default).unwrap();

        let path =  Path::new("./output.o\0");
        let result = target_machine.write_to_file(&module, FileType::Object, &path);
        println!("{:?}", result);

        module.print_to_stderr();
        // This is an object file. In order to run it as a binary,
        // we need to link it using `cc output.o -o output`.
        // Now you can run `./output`.
    }
}

