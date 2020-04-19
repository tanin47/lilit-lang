use parse::tree::{Type, Class, Expr};
use analyse::scope::Scope;
use std::borrow::Borrow;

pub fn apply<'def>(
    tpe: &Type<'def>,
    scope: &mut Scope<'def>
) {
    if tpe.class_def.get().is_none() {
        tpe.class_def.set(scope.find_class(tpe.span.unwrap().fragment).map(|c| c.parse));
    }
}


pub trait GetType<'def> {
    fn get_type(&self, scope: &Scope<'def>) -> &Class<'def>;
}

impl <'def> GetType<'def> for Expr<'def> {
    fn get_type(&self, scope: &Scope<'def>) -> &Class<'def> {
        match self {
            Expr::Identifier(i) => unsafe { &*i.source.borrow().as_ref().unwrap().get_type() },
            Expr::MemberAccess(i) => unsafe { &*(&*i.param_def.get().unwrap()).tpe.class_def.get().unwrap() },
            Expr::NewInstance(i) => unsafe { &*i.class_def.get().unwrap() },
            Expr::Int(i) => unsafe { &*scope.find_class("Int").unwrap().parse },
            Expr::String(i) => unsafe { &*scope.find_class("String").unwrap().parse },
            Expr::Char(i) => unsafe { &*scope.find_class("Char").unwrap().parse },
            Expr::NativeInt(i) => unsafe { &*scope.find_class("Native__Int").unwrap().parse },
            Expr::NativeString(i) => unsafe { &*scope.find_class("Native__String").unwrap().parse },
            Expr::NativeChar(i) => unsafe { &*scope.find_class("Native__Char").unwrap().parse },
            Expr::Invoke(i) => unsafe { &*(&*i.method_def.get().unwrap()).return_type.class_def.get().unwrap() },
            Expr::Assignment(i) => unsafe { &*i.tpe.get().unwrap() },
        }
    }
}