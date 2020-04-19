
extern crate lilit;
extern crate inkwell;

use lilit::{analyse, emit, index, parse};
use std::env;
use std::fs::File;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use inkwell::targets::{InitializationConfig, Target, TargetMachine, RelocMode, CodeModel, FileType};
use inkwell::OptimizationLevel;
use std::path::Path;


fn main() {
    println!("Lilit 0.1.0\n");
    let args: Vec<String> = env::args().collect();
    let mut file = File::open(&args[1]).expect("file not found");

    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("something went wrong reading the file");

    compile(content.trim(), &args[1]);
}

fn compile(content: &str, path: &str) {
    let mut file = parse::apply(content, path).unwrap();

    println!("---- Code ----");
    println!("{}\n", content);

    let root = index::build(&[file.deref()]);

    analyse::apply(&mut [file.deref_mut()], &root);

    let module = emit::apply(&[file.deref()]);
    module.print_to_stderr();

    Target::initialize_native(&InitializationConfig::default()).unwrap();

    let triple = TargetMachine::get_default_triple().to_string();
    let target = Target::from_triple(&triple).unwrap();
    let target_machine = target.create_target_machine(&triple, "generic", "", OptimizationLevel::None, RelocMode::Default, CodeModel::Default).unwrap();

    let output_path = Path::new("./output/main.o");

    println!("Write LLVM object to {}", output_path.display());
    target_machine.write_to_file(&module, FileType::Object, &output_path).unwrap();
}

