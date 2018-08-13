use std::fmt::{Debug, Error, Formatter};

pub enum Node {
    Num(i32),
    Func(Box<String>, Box<Node>),
    Mod(Box<Node>, Option<Box<Node>>),
    Error,
}

impl Debug for Node {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Node::*;
        match *self {
            Num(num) => write!(fmt, "Num({:?})", num),
            Func(ref ident, ref node) => write!(fmt, "Func({:?} {:?})", ident, node),
            Mod(ref func, ref opt) => write!(fmt, "Mod({:?}, {:?})", func, opt),
            Error => write!(fmt, "error"),
        }
    }
}
