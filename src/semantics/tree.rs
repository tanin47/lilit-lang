use std::cell::Cell;
use std::fmt::{Debug, Error, Formatter};
use std::marker::PhantomData;

use inkwell::types::StructType;
use inkwell::values::{FunctionValue, PointerValue};

use syntax;

#[derive(Debug)]
pub struct Mod {
    pub units: Vec<ModUnit>,
    pub number_class: Cell<Option<*const Class>>,
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
    pub methods: Vec<Func>,
    pub is_llvm: bool,
    pub llvm_struct_type_ref: Cell<Option<StructType>>
}

#[derive(Debug)]
pub struct Param {
    pub var: Box<Var>,
    pub tpe_name: String,
    pub tpe: Cell<Option<ExprType>>,
}

#[derive(Debug)]
pub struct Func {
    pub llvm_ref: Cell<Option<FunctionValue>>,
    pub parent_class_opt: Cell<Option<* const Class>>,
    pub name: String,
    pub params: Vec<Param>,
    pub return_type_name: String,
    pub return_type: Cell<Option<ExprType>>,
    pub exprs: Vec<Expr>,
}

#[derive(Debug, Copy, Clone)]
pub enum ExprType {
    Void,
    LlvmNumber,
    String,
    Boolean,
    Class(*const Class),
}

#[derive(Debug)]
pub enum Expr {
    Invoke(Box<Invoke>),
    LlvmInvoke(Box<LlvmInvoke>),
    LiteralString(Box<LiteralString>),
    Assignment(Box<Assignment>),
    Boolean(Box<Boolean>),
    IfElse(Box<IfElse>),
    ReadVar(Box<ReadVar>),
    ClassInstance(Box<ClassInstance>),
    LlvmClassInstance(Box<LlvmClassInstance>),
    DotInvoke(Box<DotInvoke>),
    DotMember(Box<DotMember>),
    LlvmNumber(Box<LlvmNumber>),
}

impl Expr {
    pub fn get_type(&self) -> ExprType {
        match self {
            Expr::Invoke(invoke) => invoke.tpe.get().unwrap(),
            Expr::LlvmInvoke(invoke) => invoke.return_type.get().unwrap(),
            Expr::LiteralString(s) => s.tpe,
            Expr::Assignment(a) => a.tpe.get().unwrap(),
            Expr::Boolean(b) => b.tpe,
            Expr::IfElse(i) => i.tpe.get().unwrap(),
            Expr::ReadVar(r) => r.tpe.get().unwrap(),
            Expr::ClassInstance(i) => i.tpe.get().unwrap(),
            Expr::LlvmClassInstance(i) => i.tpe.get().unwrap(),
            Expr::LlvmNumber(i) => ExprType::LlvmNumber,
            Expr::DotInvoke(d) => d.tpe.get().unwrap(),
            Expr::DotMember(d) => d.tpe.get().unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct DotMember {
    pub expr: Box<Expr>,
    pub member: Box<Member>,
    pub tpe: Cell<Option<ExprType>>
}

#[derive(Debug)]
pub struct Member {
    pub name: String,
    pub param_index: Cell<Option<i32>>,
    pub tpe: Cell<Option<ExprType>>
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
    pub class_ref: Cell<Option<*const Class>>,
    pub tpe: Cell<Option<ExprType>>,
}

#[derive(Debug)]
pub struct LlvmNumber {
    pub value: i32,
}

#[derive(Debug)]
pub struct LlvmClassInstance {
    pub name: String,
    pub params: Vec<Expr>,
    pub class_ref: Cell<Option<*const Class>>,
    pub tpe: Cell<Option<ExprType>>,
}

#[derive(Debug)]
pub struct IfElse {
    pub cond: Box<Expr>,
    pub true_br: Box<Expr>,
    pub false_br: Box<Expr>,
    pub tpe: Cell<Option<ExprType>>,
}

#[derive(Debug)]
pub struct LiteralString {
    pub content: String,
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
    pub return_type: Cell<Option<ExprType>>,
    pub args: Vec<Expr>,
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
    pub member_param_index: Cell<Option<i32>>,
}

#[derive(Debug)]
pub struct Var {
    pub llvm_ref: Cell<Option<PointerValue>>,
    pub tpe: Cell<Option<ExprType>>,
    pub name: String,
}
