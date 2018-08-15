use std::fmt::{Debug, Error, Formatter};


pub struct Mod {
    pub func: Box<Func>,
    pub next_opt: Option<Box<Mod>>,
}

pub struct Func {
    pub name: Box<String>,
    pub expr: Box<Num>,
}

pub struct Num {
    pub value: i32
}

impl Debug for Num {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Num({:?})", (*self).value)
    }
}

impl Debug for Func {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Func({:?}, {:?})", (*self).name, (*self).expr)
    }
}

impl Debug for Mod {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "Mod({:?}, {:?})", (*self).func, (*self).next_opt)
    }
}
