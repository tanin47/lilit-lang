use std::env;
use std::fs::File;
use std::io::prelude::*;

mod lilit;
mod ast;

fn main() {
    println!("Lilit 0.1.0");
    let args: Vec<String> = env::args().collect();
    let mut file = File::open(&args[1]).expect("file not found");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    println!("{:?}", lilit::ExprParser::new().parse(&contents));


    println!("With text:\n{}", contents);
}
