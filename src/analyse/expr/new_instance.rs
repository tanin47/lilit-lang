use parse::tree::{NewInstance, TypeKind};
use analyse::scope::Scope;
use analyse::expr;

pub fn apply<'def>(
    new_instance: &mut NewInstance<'def>,
    scope: &mut Scope<'def>,
) {
    match new_instance.name_opt {
       Some(name) => new_instance.tpe = Some(scope.find_type(name.fragment, &new_instance.generics)),
       None => (),
    };

    if let Some(TypeKind::Class(class_type)) = &new_instance.tpe {
        new_instance.generics = class_type.generics.clone()
    }

    for arg in &mut new_instance.args {
       expr::apply(arg, scope);
    }
}
