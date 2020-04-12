#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate inkwell;

use parse::tree::CompilationUnit;

#[macro_use]
pub mod macros;

#[cfg(test)]
pub mod test_common;

pub mod analyse;
pub mod emit;
pub mod index;
pub mod tokenize;
pub mod parse;

#[derive(Debug, PartialEq)]
pub struct LilitFile<'def> {
    pub unit: CompilationUnit<'def>,
    pub content: String,
    pub path: String,
}

