use std::fmt::{Debug, Error, Formatter};

pub enum Expr {
    Func(Box<String>, i32),
    Error,
}

impl Debug for Expr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Expr::*;
        match *self {
            Func(ref ident, ret_val) => write!(fmt, "({:?} {:?})", ident, ret_val),
            Error => write!(fmt, "error"),
        }
    }
}
