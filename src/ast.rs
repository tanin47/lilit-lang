use std::fmt::{Debug, Error, Formatter};
use std::rc::Rc;


pub struct Mod {
    pub units: Vec<Rc<ModUnit>>,
}

pub enum ModUnit {
    Func(Rc<Func>),
    Class(Rc<Class>),
}

pub struct Class {
    pub name: String,
    pub extends: Vec<String>,
    pub methods: Vec<Rc<Func>>,
}

pub struct Func {
    pub id: Rc<Id>,
    pub exprs: Vec<Rc<Expr>>,
}

pub enum Expr {
    Invoke(Rc<Invoke>),
    Num(Rc<Num>),
    Assignment(Rc<Assignment>),
    Var(Rc<Var>),
}

pub struct Assignment {
    pub var: Rc<Var>,
    pub expr: Rc<Expr>,
}

pub struct Var {
    pub id: Rc<Id>,
}

pub struct Id {
    pub name: String
}

pub struct Invoke {
    pub id: Rc<Id>,
}

pub struct Num {
    pub value: i32
}

impl Debug for Id {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Id({:?})", (*self).name)
    }
}

impl Debug for Expr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Expr::Num(n) => write!(fmt, "Num({:?})", n.value),
            Expr::Invoke(i) => write!(fmt, "Invoke({:?})", i.id),
            Expr::Assignment(a) => write!(fmt, "Assignment({:?}, {:?})", a.var, a.expr),
            Expr::Var(v) => write!(fmt, "Var({:?})", v.id.name),
        }
    }
}

impl Debug for Var {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Var({:?})", (*self).id.name)
    }
}


impl Debug for Func {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Func({:?}, {:?})", (*self).id.name, (*self).exprs)
    }
}

impl Debug for Class {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Class({:?}, {:?}, {:?})", (*self).name, (*self).extends, (*self).methods)
    }
}

impl Debug for ModUnit {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            ModUnit::Class(c) => write!(fmt, "ClassUnit({:?})", c),
            ModUnit::Func(f) => write!(fmt, "FuncUnit({:?})", f),
        }
    }
}

impl Debug for Mod {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Mod({:?})", (*self).units)
    }
}
