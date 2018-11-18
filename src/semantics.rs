use std::fmt::{Debug, Error, Formatter};
use std::marker::PhantomData;
use std::cell::RefCell;
use ast;
use inkwell::values::{FunctionValue, PointerValue};
use std::rc::Rc;


pub struct Mod<'a> {
    pub units: Vec<Rc<ModUnit<'a>>>,
    pub syntax: Rc<ast::Mod>,
}

pub enum ModUnit<'a> {
    Func {
    	func: Rc<Func<'a>>,
    	syntax: Rc<ast::ModUnit>,
    },
    Class {
    	class: Rc<Class<'a>>,
    	syntax: Rc<ast::ModUnit>,
    },
}

pub struct Class<'a> {
    pub extends: Vec<Rc<Class<'a>>>,
    pub methods: Vec<Rc<Func<'a>>>,
    pub syntax: Rc<ast::Class>,
}

pub struct Func<'a> {
	pub llvm_ref: RefCell<Option<FunctionValue>>,
    pub exprs: Vec<Rc<Expr<'a>>>,
    pub syntax: Rc<ast::Func>,
}

pub enum Expr<'a> {
    Invoke {
    	invoke: Rc<Invoke<'a>>,
    	syntax: Rc<ast::Expr>,
    },
    Num {
    	num: Rc<Num>,
    	syntax: Rc<ast::Expr>,
    },
    Assignment {
        assignment: Rc<Assignment<'a>>,
        syntax: Rc<ast::Expr>,
    },
    ReadVar {
        read_var: Rc<ReadVar>,
        syntax: Rc<ast::Expr>,
    }
}

pub struct Assignment<'a> {
    pub var: Rc<Var>,
    pub expr: Rc<Expr<'a>>,
    pub syntax: Rc<ast::Assignment>,
}

pub struct ReadVar {
    pub origin: Rc<Var>,
    pub syntax: Rc<ast::Var>,
}

pub struct Var {
    pub llvm_ref: RefCell<Option<PointerValue>>,
    pub id: Rc<Id>,
    pub syntax: Rc<ast::Var>,
}

pub struct Id {
    pub syntax: Rc<ast::Id>,
}

pub struct Invoke<'a> {
	pub func_opt: RefCell<Option<Rc<Func<'a>>>>,
	pub syntax: Rc<ast::Invoke>,
}

pub struct Num {
	pub value: i32,
	pub syntax: Rc<ast::Num>,
}

impl Debug for Id {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Id({:?})", (*self).syntax.name)
    }
}

impl<'a> Debug for Invoke<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Invoke({:?})", (*self).func_opt)
    }
}

impl Debug for Var {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Var({:?})", (*self).id)
    }
}

impl Debug for ReadVar {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "ReadVar({:?})", (*self).syntax.id.name)
    }
}

impl<'a> Debug for Assignment<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Assignment({:?}, {:?})", (*self).var, (*self).expr)
    }
}

impl Debug for Num {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Num({:?})", (*self).value)
    }
}

impl<'a> Debug for Expr<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Expr::Num { num, syntax: _ } => write!(fmt, "{:?}", num),
            Expr::Invoke { invoke, syntax: _ } => write!(fmt, "{:?}", invoke),
            Expr::Assignment { assignment, syntax: _ } => write!(fmt, "{:?}", assignment),
            Expr::ReadVar { read_var, syntax: _ } => write!(fmt, "{:?}", read_var),
        }
    }
}

impl<'a> Debug for Func<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Func({:?}, {:?})", (*self).syntax.id, (*self).exprs)
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
