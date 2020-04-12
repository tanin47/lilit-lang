use parse::tree::Expr;
use analyse::scope::Scope;

pub mod identifier;
pub mod int;
pub mod invoke;
pub mod literal_string;
pub mod member_access;
pub mod new_instance;

pub fn apply<'def>(
    expr: &Expr<'def>,
    scope: &mut Scope<'def>,
) {
    match expr {
        Expr::Invoke(e) => invoke::apply(e, scope),
        Expr::Int(e) => int::apply(e, scope),
        Expr::String(e) => literal_string::apply(e, scope),
        Expr::Identifier(e) => identifier::apply(e, scope),
        Expr::MemberAccess(e) => member_access::apply(e, scope),
        Expr::NewInstance(e) => new_instance::apply(e, scope),
        other => panic!("Unsupported expr {:#?}", other),
    }
}
