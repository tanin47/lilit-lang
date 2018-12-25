#![allow(warnings)]

use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::collections::HashMap;

extern crate inkwell;

use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, BasicValue, IntValue};
use inkwell::basic_block::BasicBlock;
use inkwell::targets::{InitializationConfig, Target, TargetMachine, RelocMode, CodeModel, FileType};

mod syntax;
mod semantics;
mod llvmgen;

fn main() {
    println!("Lilit 0.0.1\n");
    let args: Vec<String> = env::args().collect();
    let mut file = File::open(&args[1]).expect("file not found");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    let tree = syntax::ModParser::new().parse(&contents);

    println!("---- Code ----");
    println!("{}\n", contents);


    println!("{:?}\n", tree);

    // The first pass makes a hashtable for function and class.

    if let Ok(ref _ok_tree) = tree {
        let mut root = Box::new(semantics::analyse(_ok_tree));
        println!("{:?}\n", root);

        Target::initialize_native(&InitializationConfig::default()).unwrap();

        let context = Context::create();
        let builder = context.create_builder();

        println!("Start generating LLVM IR...");
        let module = llvmgen::generate(&root, &context, &builder);
        println!("Finished");
        module.print_to_stderr();

        let triple = TargetMachine::get_default_triple().to_string();
        let target = Target::from_triple(&triple).unwrap();
        let target_machine = target.create_target_machine(&triple, "generic", "", OptimizationLevel::None, RelocMode::Default, CodeModel::Default).unwrap();

        let path =  Path::new("./output/main.o");
        println!("Write LLVM IR to main.o");
        let result = target_machine.write_to_file(&module, FileType::Object, &path);
         // This is an object file. In order to run it as a binary,
         // we need to link it using `cc output.o -o output`.
         // Now you can run `./output`.
    }
}

