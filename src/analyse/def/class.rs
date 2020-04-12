use parse::tree::{Class, ParamParent};
use analyse::def::{method, params};
use analyse::scope::Scope;

pub fn apply<'def>(
    class: &Class<'def>,
    scope: &mut Scope<'def>,
) {
    scope.enter_class(class);

    params::apply(&class.params, ParamParent::Class(class), scope);

    for m in &class.methods {
        method::apply(m, scope);
    }
    scope.leave();
}
