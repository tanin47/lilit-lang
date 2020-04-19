use parse::tree::{Param, ParamParent};
use analyse::scope::Scope;
use analyse::tpe;

pub fn apply<'def>(
    params: &mut Vec<Param<'def>>,
    parent: ParamParent<'def>,
    scope: &mut Scope<'def>
) {
    for (index, param) in params.iter_mut().enumerate() {
        tpe::apply(&mut param.tpe, scope);
        param.parent = Some(parent);
        param.index = index;
    }
}
