use std::fmt::{Debug, Error, Formatter};


pub struct Mod {
    pub units: Vec<Box<ModUnit>>,
}

pub enum ModUnit {
    FuncUnit(Box<Func>),
    ClassUnit(Box<Class>),
}

pub struct Class {
    pub name: Box<String>,
    pub extends: Vec<Box<String>>,
    pub methods: Vec<Box<Func>>,
}

pub struct Func {
    pub name: Box<String>,
    pub exprs: Vec<Box<Expr>>,
}

pub enum Expr {
    Invoke { name: Box<String> },
    Num { value: i32 },
}


impl Debug for Expr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Expr::Num { value } => write!(fmt, "Num({:?})", value),
            Expr::Invoke { name } => write!(fmt, "Invoke({:?})", name),
        }
    }
}

impl Debug for Func {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Func({:?}, {:?})", (*self).name, (*self).exprs)
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
            ModUnit::ClassUnit(c) => write!(fmt, "ClassUnit({:?})", c),
            ModUnit::FuncUnit(f) => write!(fmt, "FuncUnit({:?})", f),
        }
    }
}

impl Debug for Mod {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Mod({:?})", (*self).units)
    }
}
