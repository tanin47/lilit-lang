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
use inkwell::targets::{InitializationConfig, Target, TargetMachine, RelocMode, CodeModel, FileType};

mod lilit;
mod ast;

fn main() {
    println!("Lilit 0.0.1");
    let args: Vec<String> = env::args().collect();
    let mut file = File::open(&args[1]).expect("file not found");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    println!("{:?}", lilit::ExprParser::new().parse(&contents));


    println!("With text:\n{}", contents);

    Target::initialize_native(&InitializationConfig::default()).unwrap();

    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();

    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);

    let function = module.add_function("main", &fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    let ret = i32_type.const_int(123, false);

    builder.build_return(Some(&ret));

    let triple = TargetMachine::get_default_triple().to_string();
    let target = Target::from_triple(&triple).unwrap();
    let target_machine = target.create_target_machine(&triple, "generic", "", OptimizationLevel::Default, RelocMode::Default, CodeModel::Default).unwrap();

    let path =  Path::new("./output.o");
    target_machine.write_to_file(&module, FileType::Object, &path);
    // This is an object file. In order to run it as a binary,
    // we need to link it using `cc output.o -o output`.
    // Now you can run `./output`.
}

