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
    pub params: Vec<Box<Param>>,
    pub extends: Vec<String>,
    pub methods: Vec<Box<Func>>,
    pub is_llvm: bool,
}

#[derive(Debug)]
pub struct Param {
    pub var: Box<Var>,
    pub tpe: String
}

#[derive(Debug)]
pub struct Func {
    pub name: String,
    pub params: Vec<Box<Param>>,
    pub return_type: String,
    pub exprs: Vec<Box<Expr>>,
}

#[derive(Debug)]
pub enum Expr {
    Invoke(Box<Invoke>),
    LlvmInvoke(Box<LlvmInvoke>),
    Boolean(Box<Boolean>),
    Num(Box<Num>),
    Assignment(Box<Assignment>),
    Var(Box<Var>),
    LiteralString(Box<LiteralString>),
    IfElse(Box<IfElse>),
    ClassInstance(Box<ClassInstance>),
    LlvmClassInstance(Box<LlvmClassInstance>),
    DotInvoke(Box<DotInvoke>),
    DotMember(Box<DotMember>),
    Array(Box<Array>),
}

#[derive(Debug)]
pub struct Array {
    pub items: Vec<Box<Expr>>,
}

#[derive(Debug)]
pub struct DotMember {
    pub expr: Box<Expr>,
    pub member: Box<Var>,
}

#[derive(Debug)]
pub struct DotInvoke {
    pub expr: Box<Expr>,
    pub invoke: Box<Invoke>,
}

#[derive(Debug)]
pub struct ClassInstance {
    pub name: String,
    pub params: Vec<Box<Expr>>,
}

#[derive(Debug)]
pub struct LlvmClassInstance {
    pub name: String,
    pub params: Vec<Box<Expr>>,
}

#[derive(Debug)]
pub struct IfElse {
    pub cond: Box<Expr>,
    pub true_br: Box<Expr>,
    pub false_br: Box<Expr>,
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
    pub args: Vec<Box<Expr>>,
}

#[derive(Debug)]
pub struct LlvmInvoke {
    pub name: String,
    pub is_varargs: bool,
    pub return_type: String,
    pub args: Vec<Box<Expr>>,
}

#[derive(Debug)]
pub struct Num {
    pub value: i32,
}

#[derive(Debug)]
pub struct Boolean {
    pub value: bool,
}
