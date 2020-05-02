use parse::tree::{Type, Class, Expr, TypeKind};
use analyse::scope::Scope;
use std::borrow::Borrow;

pub fn apply<'def>(
    tpe: &mut Type<'def>,
    scope: &mut Scope<'def>
) {
    match tpe.kind.as_ref() {
        TypeKind::Class(class_type) => {
            if class_type.class_def.is_none() { // Type has never been analysed.
                tpe.kind = Box::new(scope.find_type(tpe.span.unwrap().fragment, &class_type.generics));
            }
        },
        TypeKind::Generic(g) => {}, // do nothing. Type has been already analysed.
    };
}


pub trait GetType<'def> {
    fn get_type(&self, scope: &Scope<'def>) -> TypeKind<'def>;
}

impl <'def> GetType<'def> for Expr<'def> {
    fn get_type(&self, scope: &Scope<'def>) -> TypeKind<'def> {
        match self {
            Expr::Identifier(i) => unsafe { &*i.source.borrow().as_ref().unwrap().get_type() }.clone(),
            Expr::MemberAccess(i) => unsafe { &*i.param_def.unwrap() }.tpe.kind.as_ref().clone(),
            Expr::NewInstance(i) => i.tpe.as_ref().unwrap().clone(),
            Expr::Int(i) => scope.find_type("Int", &[]),
            Expr::String(i) => scope.find_type("String", &[]),
            Expr::Char(i) => scope.find_type("Char", &[]),
            Expr::NativeInt(i) => scope.find_type("Native__Int", &[]),
            Expr::NativeString(i) => scope.find_type("Native__String", &[]),
            Expr::NativeChar(i) => scope.find_type("Native__Char", &[]),
            Expr::Invoke(i) => {
                if let Some(invoker) = i.invoker_opt.as_ref() {
                    let parent_tpe = invoker.get_type(scope);
                    // We have the parent type. We need to resolve the return type against the parent type for a generic type
                }
                unsafe { &*i.method_def.unwrap() }.return_type.kind.as_ref().clone()
            },
            Expr::Assignment(i) => i.tpe.as_ref().unwrap().clone(),
        }
    }
}
