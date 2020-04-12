use parse::tree::{Identifier, Param};
use analyse::scope::Scope;

pub fn apply<'def>(
    identifier: &Identifier<'def>,
    scope: &mut Scope<'def>,
) {
    identifier.def_opt.set(Some(scope.find_identifier(identifier.name.fragment).unwrap()));
}
