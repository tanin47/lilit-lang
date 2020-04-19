use parse::tree::{Method, ParamParent, Class, Param, Type};
use analyse::{expr, tpe};
use analyse::scope::Scope;
use analyse::def::params;
use std::cell::Cell;

pub fn apply<'def>(
    method: &mut Method<'def>,
    parent_class: Option<*const Class<'def>>,
    scope: &mut Scope<'def>
) {
    scope.enter_method(method);

    if let Some(parent_class) = parent_class {
        method.params.insert(0, Param {
            name: None,
            tpe: Type {
                span: None,
                def_opt: Cell::new(Some(parent_class)),
            },
            is_varargs: false,
            index: 0,
            parent: Cell::new(Some(ParamParent::Method(method))),
            llvm: Cell::new(None)
        })
    }

    let parent = ParamParent::Method(method);
    params::apply(&mut method.params, parent, scope);
    tpe::apply(&method.return_type, scope);

    for e in &method.exprs {
        expr::apply(e, scope);
    }
    scope.leave();
}
