use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    println!("Lilit 0.1.0");
    let args: Vec<String> = env::args().collect();
    let mut f = File::open(&args[1]).expect("file not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    println!("With text:\n{}", contents);
}
