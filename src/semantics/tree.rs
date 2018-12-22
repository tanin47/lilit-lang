use std::cell::Cell;
use std::fmt::{Debug, Error, Formatter};
use std::marker::PhantomData;

use inkwell::types::StructType;
use inkwell::values::{FunctionValue, PointerValue};

use llvmgen::native::gen::NativeTypeEnum;
use syntax;

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
pub struct LlvmClass {
    pub tpe: NativeTypeEnum,
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub params: Vec<Box<ClassParam>>,
    pub extends: Vec<String>,
    pub methods: Vec<Func>,
    pub llvm_struct_type_ref: Cell<Option<StructType>>
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
    LlvmClass(*const LlvmClass),
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
    DotMember(Box<DotMember>),
}

impl Expr {
    pub fn get_type(&self) -> ExprType {
        match self {
            Expr::Invoke(invoke) => invoke.tpe.get().unwrap(),
            Expr::LlvmInvoke(invoke) => invoke.return_type.tpe.get_expr_type(),
            Expr::Num(num) => num.tpe,
            Expr::LiteralString(s) => s.tpe,
            Expr::Assignment(a) => a.tpe.get().unwrap(),
            Expr::Boolean(b) => b.tpe,
            Expr::Comparison(c) => c.tpe,
            Expr::IfElse(i) => i.tpe.get().unwrap(),
            Expr::ReadVar(r) => r.tpe.get().unwrap(),
            Expr::ClassInstance(i) => i.tpe.get().unwrap(),
            Expr::LlvmClassInstance(i) => ExprType::LlvmClass(i.class.as_ref()),
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
pub struct LlvmClassInstance {
    pub expr: Box<Expr>,
    pub class: Box<LlvmClass>,
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
    pub return_type: LlvmClass,
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
