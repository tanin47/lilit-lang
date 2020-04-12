use parse::tree::{Method, ParamParent};
use analyse::{expr, tpe};
use analyse::scope::Scope;
use analyse::def::params;

pub fn apply<'def>(
    method: &Method<'def>,
    scope: &mut Scope<'def>
) {
    scope.enter_method(method);

    params::apply(&method.params, ParamParent::Method(method), scope);
    tpe::apply(&method.return_type, scope);

    for e in &method.exprs {
        expr::apply(e, scope);
    }
    scope.leave();
}
