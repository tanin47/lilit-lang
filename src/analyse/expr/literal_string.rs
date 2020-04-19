use parse::tree::{LiteralString, NewInstance, Expr, NativeString};
use analyse::scope::Scope;
use std::cell::Cell;

pub fn apply<'def>(
    string: &LiteralString<'def>,
    scope: &mut Scope<'def>,
) {
    string.instance.replace(Some(NewInstance {
        name_opt: None,
        args: vec![
            Expr::NewInstance(Box::new(NewInstance {
                name_opt: None,
                args: vec![
                    Expr::NativeString(Box::new(NativeString { value: serde_json::from_str(string.span.fragment).unwrap() }))
                ],
                class_def: Cell::new(Some(scope.find_class("Native__String").unwrap().parse))
            })),
        ],
        class_def: Cell::new(Some(scope.find_class("String").unwrap().parse))
    }));
}

#[cfg(test)]
mod tests {
    use std::ops::{Deref, DerefMut};

    use index::build;
    use parse;
    use test_common::span2;
    use analyse::apply;
    use std::cell::{Cell, RefCell};
    use parse::tree::{LiteralString, Expr, NewInstance, NativeString};

    #[test]
    fn test_simple() {
        let content = r#"
class Void
end

class Native__String
end

class String(underlying: Native__String)
end

def main: Void
  "test"
end
        "#;
        let mut file = unwrap!(Ok, parse::apply(content.trim(), ""));
        let root = build(&[file.deref()]);

        apply(&mut [file.deref_mut()], &root);

        assert_eq!(
            root.find_method("main").exprs.get(0).unwrap(),
            &Expr::String(Box::new(LiteralString {
                span: span2(11, 3, "\"test\"", file.deref()),
                instance: RefCell::new(Some(
                    NewInstance {
                        name_opt: None,
                        args: vec![
                            Expr::NewInstance(Box::new(NewInstance {
                                name_opt: None,
                                args: vec![
                                    Expr::NativeString(Box::new(NativeString { value: "test".to_string() }))
                                ],
                                class_def: Cell::new(Some(root.find_class("Native__String")))
                            })),
                        ],
                        class_def: Cell::new(Some(root.find_class("String")))
                    }
                ))
            }))
        )
    }
}
