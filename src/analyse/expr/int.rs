use parse::tree::{Int, NewInstance, NativeInt, Expr, ClassType, TypeKind};
use analyse::scope::Scope;
use std::cell::Cell;

pub fn apply<'def>(
    int: &mut Int<'def>,
    scope: &mut Scope<'def>,
) {
    int.instance = Some(Box::new(NewInstance {
        name_opt: None,
        generics: vec![],
        args: vec![
            Expr::NewInstance(Box::new(NewInstance {
                name_opt: None,
                generics: vec![],
                args: vec![
                    Expr::NativeInt(Box::new(NativeInt { value: int.span.fragment.parse::<i64>().unwrap() }))
                ],
                tpe: Some(TypeKind::Class(ClassType { class_def: Some(scope.find_class("Native__Int").unwrap().parse), generics: vec![] }))
            })),
        ],
        tpe: Some(TypeKind::Class(ClassType { class_def: Some(scope.find_class("Int").unwrap().parse), generics: vec![] }))
    }));
}

#[cfg(test)]
mod tests {
    use std::ops::{Deref, DerefMut};

    use index::build;
    use parse;
    use parse::tree::{CompilationUnit, Type, CompilationUnitItem, Method, Invoke, Expr, Int, NewInstance, NativeInt, TypeKind};
    use test_common::span2;
    use analyse::apply;
    use std::cell::{Cell, RefCell};

    #[test]
    fn test_simple() {
        let content = r#"
class Void
end

class Native__Int
end

class Int(underlying: Native__Int)
end

def main: Void
  1
end
        "#;
        let mut file = unwrap!(Ok, parse::apply(content.trim(), ""));
        let root = build(&[file.deref()]);

        apply(&mut [file.deref_mut()], &root);

        assert_eq!(
            root.find_method("main").exprs.get(0).unwrap(),
            &Expr::Int(Box::new(Int {
                span: span2(11, 3, "1", file.deref()),
                instance: Some(Box::new(
                  NewInstance {
                      name_opt: None,
                      generics: vec![],
                      args: vec![
                          Expr::NewInstance(Box::new(NewInstance {
                              name_opt: None,
                              generics: vec![],
                              args: vec![
                                  Expr::NativeInt(Box::new(NativeInt { value: 1 }))
                              ],
                              tpe: Some(TypeKind::init_class_type(root.find_class("Native__Int")))
                          })),
                      ],
                      tpe: Some(TypeKind::init_class_type(root.find_class("Int")))
                  }
                ))
            }))
        )
    }
}
