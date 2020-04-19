use parse::tree::{Class, ParamParent};
use analyse::def::{method, params};
use analyse::scope::Scope;
use std::ops::Deref;

pub fn apply<'def>(
    class: &mut Class<'def>,
    scope: &mut Scope<'def>,
) {
    scope.enter_class(class);

    let parent = class as *const Class<'def>;
    params::apply(&mut class.params, ParamParent::Class(parent), scope);

    for m in &mut class.methods {
        method::apply(m, Some(parent), scope);
    }
    scope.leave();
}
