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
    pub exprs: Vec<Expr>,
}

#[derive(Debug)]
pub enum Expr {
    Invoke {
        invoke: Box<Invoke>,
    },
    Num {
        num: Box<Num>,
    },
    LiteralString {
        literal_string: Box<LiteralString>,
    },
    Assignment {
        assignment: Box<Assignment>,
    },
    Boolean {
        boolean: Box<Boolean>,
    },
    Comparison {
        comparison: Box<Comparison>,
    },
    IfElse {
        if_else: Box<IfElse>,
    },
    ReadVar {
        read_var: Box<ReadVar>,
    },
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
pub struct Invoke {
    pub func_ref: Cell<Option<*const Func>>,
    pub name: String,
    pub arg: Box<Expr>,
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
    pub name: String,
}
