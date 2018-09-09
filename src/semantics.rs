use std::fmt::{Debug, Error, Formatter};
use std::cell::Cell;
use ast;


pub struct Mod<'a> {
    pub units: Vec<ModUnit<'a>>,
    pub syntax: &'a ast::Mod,
}

pub enum ModUnit<'a> {
    Func {
    	func: Box<Func<'a>>,
    	syntax: &'a ast::ModUnit,
    },
    Class {
    	class: Box<Class<'a>>,
    	syntax: &'a ast::ModUnit,
    },
}

pub struct Class<'a> {
    pub extends: Vec<Class<'a>>,
    pub methods: Vec<Func<'a>>,
    pub syntax: &'a ast::Class,
}

pub struct Func<'a> {
    pub exprs: Vec<Expr<'a>>,
    pub syntax: &'a ast::Func,
}

pub enum Expr<'a> {
    Invoke {
    	invoke: Box<Invoke<'a>>,
    	syntax: &'a ast::Expr,
    },
    Num {
    	num: Box<Num<'a>>,
    	syntax: &'a ast::Expr,
    },
}

pub struct Invoke<'a> {
	pub func_opt: Cell<Option<&'a Func<'a>>>,
	pub syntax: &'a ast::Invoke,
}

pub struct Num<'a> {
	pub value: i32,
	pub syntax: &'a ast::Num,
}

impl<'a> Debug for Invoke<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Invoke({:?})", (*self).func_opt)
    }
}

impl<'a> Debug for Num<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Num({:?})", (*self).value)
    }
}

impl<'a> Debug for Expr<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Expr::Num { num, syntax: _ } => write!(fmt, "{:?}", num),
            Expr::Invoke { invoke, syntax: _ } => write!(fmt, "{:?}", invoke),
        }
    }
}

impl<'a> Debug for Func<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Func({:?}, {:?})", (*self).syntax.name, (*self).exprs)
    }
}

impl<'a> Debug for Class<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Class({:?}, {:?}, {:?})", (*self).syntax.name, (*self).extends, (*self).methods)
    }
}

impl<'a> Debug for ModUnit<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            ModUnit::Class { class, syntax: _ } => write!(fmt, "{:?}", class),
            ModUnit::Func { func, syntax: _ } => write!(fmt, "{:?}", func),
        }
    }
}

impl<'a> Debug for Mod<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Mod({:?})", (*self).units)
    }
}
