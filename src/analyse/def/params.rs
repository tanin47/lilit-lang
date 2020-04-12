use parse::tree::{Param, ParamParent};
use analyse::scope::Scope;
use analyse::tpe;

pub fn apply<'def>(
    params: &Vec<Param<'def>>,
    parent: ParamParent<'def>,
    scope: &mut Scope<'def>
) {
    for (index, param) in params.iter().enumerate() {
        tpe::apply(&param.tpe, scope);
        param.parent.set(Some(parent));
    }
}
