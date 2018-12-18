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
    pub name:  String,
    pub extends: Vec<String>,
    pub methods: Vec<Func>,
}

#[derive(Debug)]
pub struct Func {
    pub llvm_ref: Cell<Option<FunctionValue>>,
    pub name: String,
    pub args: Vec<Var>,
    pub return_type: String,
    pub exprs: Vec<Expr>,
}

#[derive(Debug, Copy, Clone)]
pub enum ExprType {
    Num,
    String,
    Boolean,
    Class(* const ClassInstance),
    LlvmClass(* const LlvmClassInstance),
}

#[derive(Debug)]
pub enum Expr {
    Invoke {
        invoke: Box<Invoke>,
        tpe: Cell<Option<ExprType>>,
    },
    LlvmInvoke {
        invoke: Box<LlvmInvoke>,
        tpe: Cell<Option<ExprType>>,
    },
    Num {
        num: Box<Num>,
        tpe: Cell<Option<ExprType>>,
    },
    LiteralString {
        literal_string: Box<LiteralString>,
        tpe: Cell<Option<ExprType>>,
    },
    Assignment {
        assignment: Box<Assignment>,
        tpe: Cell<Option<ExprType>>,
    },
    Boolean {
        boolean: Box<Boolean>,
        tpe: Cell<Option<ExprType>>,
    },
    Comparison {
        comparison: Box<Comparison>,
        tpe: Cell<Option<ExprType>>,
    },
    IfElse {
        if_else: Box<IfElse>,
        tpe: Cell<Option<ExprType>>,
    },
    ReadVar {
        read_var: Box<ReadVar>,
        tpe: Cell<Option<ExprType>>,
    },
    ClassInstance {
        class_instance: Box<ClassInstance>,
        tpe: Cell<Option<ExprType>>,
    },
    LlvmClassInstance {
        class_instance: Box<LlvmClassInstance>,
        tpe: Cell<Option<ExprType>>,
    },
}

#[derive(Debug)]
pub struct ClassInstance {
    pub name: String,
    pub expr: Box<Expr>,
}

#[derive(Debug)]
pub struct LlvmClassInstance {
    pub name: String,
    pub expr: Box<Expr>,
}

#[derive(Debug)]
pub struct IfElse {
    pub cond: Box<Comparison>,
    pub true_br: Box<Expr>,
    pub false_br: Box<Expr>,
}

#[derive(Debug)]
pub struct Comparison {
    pub left: Box<ReadVar>,
    pub right: Box<Num>,
}

#[derive(Debug)]
pub struct LiteralString {
    pub content: String,
}

#[derive(Debug)]
pub struct Num {
    pub value: i32,
}

#[derive(Debug)]
pub struct Boolean {
    pub value: bool,
}

#[derive(Debug)]
pub struct LlvmInvoke {
    pub name: String,
    pub is_varargs: bool,
    pub return_type: String,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
pub struct Invoke {
    pub func_ref: Cell<Option<*const Func>>,
    pub name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
pub struct Assignment {
    pub var: Box<Var>,
    pub expr: Box<Expr>,
}

#[derive(Debug)]
pub struct ReadVar {
    pub assignment_ref: Cell<Option<*const Var>>,
    pub name: String,
}

#[derive(Debug)]
pub struct Var {
    pub llvm_ref: Cell<Option<PointerValue>>,
    pub expr_type_ref: Cell<Option<*const ExprType>>,
    pub name: String,
}
