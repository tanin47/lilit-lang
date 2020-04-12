use parse::tree::{Expr, Invoke};
use analyse::scope::Scope;
use analyse::expr;

pub fn apply<'def>(
    invoke: &Invoke<'def>,
    scope: &mut Scope<'def>,
) {
    for arg in &invoke.args {
        expr::apply(arg, scope);
    }
    match invoke.invoker_opt {
        Some(_) => (),
        None => invoke.def_opt.set(scope.find_method(invoke.name.fragment).map(|m|m.parse)),
    }
}
