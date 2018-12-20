use std::fmt::{Debug, Error, Formatter};
use std::marker::PhantomData;
use std::cell::Cell;
use syntax;
use inkwell::values::{FunctionValue, PointerValue};

#[derive(Debug)]
pub struct Mod {
    pub units: Vec<ModUnit>,
}

#[derive(Debug)]
pub enum ModUnit {
    Func {
    	func: Box<Func>,
    },
    Class {
        class: Box<Class>,
    }
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub params: Vec<Box<ClassParam>>,
    pub extends: Vec<String>,
    pub methods: Vec<Func>,
}

#[derive(Debug)]
pub struct ClassParam {
    pub var: Box<Var>,
    pub tpe_name: String,
    pub tpe: Cell<Option<ExprType>>,
}

#[derive(Debug)]
pub struct Func {
    pub llvm_ref: Cell<Option<FunctionValue>>,
    pub parent_class_opt: Cell<Option<* const Class>>,
    pub name: String,
    pub args: Vec<Var>,
    pub return_type_name: String,
    pub return_type: Cell<Option<ExprType>>,
    pub exprs: Vec<Expr>,
}

#[derive(Debug, Copy, Clone)]
pub enum ExprType {
    Void,
    Number,
    String,
    Boolean,
    Class(*const Class),
}

#[derive(Debug)]
pub enum Expr {
    Invoke(Box<Invoke>),
    LlvmInvoke(Box<LlvmInvoke>),
    Num(Box<Num>),
    LiteralString(Box<LiteralString>),
    Assignment(Box<Assignment>),
    Boolean(Box<Boolean>),
    Comparison(Box<Comparison>),
    IfElse(Box<IfElse>),
    ReadVar(Box<ReadVar>),
    ClassInstance(Box<ClassInstance>),
    LlvmClassInstance(Box<LlvmClassInstance>),
    DotInvoke(Box<DotInvoke>),
}

impl Expr {
    pub fn get_type(&self) -> ExprType {
        match self {
            Expr::Invoke(invoke) => invoke.tpe.get().unwrap(),
            Expr::LlvmInvoke(invoke) => invoke.tpe.get().unwrap(),
            Expr::Num(num) => num.tpe,
            Expr::LiteralString(s) => s.tpe,
            Expr::Assignment(a) => a.tpe.get().unwrap(),
            Expr::Boolean(b) => b.tpe,
            Expr::Comparison(c) => c.tpe,
            Expr::IfElse(i) => i.tpe.get().unwrap(),
            Expr::ReadVar(r) => r.tpe.get().unwrap(),
            Expr::ClassInstance(i) => i.tpe.get().unwrap(),
            Expr::LlvmClassInstance(i) => i.tpe.get().unwrap(),
            Expr::DotInvoke(d) => d.tpe.get().unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct DotInvoke {
    pub expr: Box<Expr>,
    pub invoke: Box<Invoke>,
    pub tpe: Cell<Option<ExprType>>
}

#[derive(Debug)]
pub struct ClassInstance {
    pub name: String,
    pub params: Vec<Box<Expr>>,
    pub tpe: Cell<Option<ExprType>>,
}

#[derive(Debug)]
pub struct LlvmClassInstance {
    pub name: String,
    pub expr: Box<Expr>,
    pub tpe: Cell<Option<ExprType>>,
}

#[derive(Debug)]
pub struct IfElse {
    pub cond: Box<Comparison>,
    pub true_br: Box<Expr>,
    pub false_br: Box<Expr>,
    pub tpe: Cell<Option<ExprType>>,
}

#[derive(Debug)]
pub struct Comparison {
    pub left: Box<ReadVar>,
    pub right: Box<Num>,
    pub tpe: ExprType,
}

#[derive(Debug)]
pub struct LiteralString {
    pub content: String,
    pub tpe: ExprType,
}

#[derive(Debug)]
pub struct Num {
    pub value: i32,
    pub tpe: ExprType,
}

#[derive(Debug)]
pub struct Boolean {
    pub value: bool,
    pub tpe: ExprType,
}

#[derive(Debug)]
pub struct LlvmInvoke {
    pub name: String,
    pub is_varargs: bool,
    pub return_type: String,
    pub args: Vec<Expr>,
    pub tpe: Cell<Option<ExprType>>,
}

#[derive(Debug)]
pub struct Invoke {
    pub func_ref: Cell<Option<*const Func>>,
    pub name: String,
    pub args: Vec<Expr>,
    pub tpe: Cell<Option<ExprType>>,
}

#[derive(Debug)]
pub struct Assignment {
    pub var: Box<Var>,
    pub expr: Box<Expr>,
    pub tpe: Cell<Option<ExprType>>,
}

#[derive(Debug)]
pub struct ReadVar {
    pub assignment_ref: Cell<Option<*const Var>>,
    pub name: String,
    pub tpe: Cell<Option<ExprType>>,
}

#[derive(Debug)]
pub struct Var {
    pub llvm_ref: Cell<Option<PointerValue>>,
    pub tpe: Cell<Option<ExprType>>,
    pub name: String,
}
