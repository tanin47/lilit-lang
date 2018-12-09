use std::fmt::{Debug, Error, Formatter};

#[derive(Debug)]
pub struct Mod {
    pub units: Vec<Box<ModUnit>>,
}

#[derive(Debug)]
pub enum ModUnit {
    Func(Box<Func>),
    Class(Box<Class>),
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub extends: Vec<String>,
    pub methods: Vec<Box<Func>>,
}

#[derive(Debug)]
pub struct Func {
    pub name: String,
    pub exprs: Vec<Box<Expr>>,
}

#[derive(Debug)]
pub enum Expr {
    Invoke(Box<Invoke>),
    Num(Box<Num>),
    Assignment(Box<Assignment>),
    Var(Box<Var>),
    LiteralString(Box<LiteralString>),
}

#[derive(Debug)]
pub struct LiteralString {
    pub content: String,
}

#[derive(Debug)]
pub struct Assignment {
    pub var: Box<Var>,
    pub expr: Box<Expr>,
}

#[derive(Debug)]
pub struct Var {
    pub name: String,
}


#[derive(Debug)]
pub struct Invoke {
    pub name: String,
    pub arg: Box<Expr>,
}

#[derive(Debug)]
pub struct Num {
    pub value: i32,
}
