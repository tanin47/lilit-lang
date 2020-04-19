use parse::tree::{NewInstance, Expr, NativeChar, Char};
use analyse::scope::Scope;
use std::cell::Cell;
use tokenize::span::CharAt;

pub fn apply<'def>(
    char: &Char<'def>,
    scope: &mut Scope<'def>,
) {
    char.instance.replace(Some(NewInstance {
        name_opt: None,
        args: vec![
            Expr::NewInstance(Box::new(NewInstance {
                name_opt: None,
                args: vec![
                    Expr::NativeChar(Box::new(NativeChar { value: char.span.fragment.char_at(1) }))
                ],
                def_opt: Cell::new(Some(scope.find_class("Native__Char").unwrap().parse))
            })),
        ],
        def_opt: Cell::new(Some(scope.find_class("Char").unwrap().parse))
    }));
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use index::build;
    use parse;
    use test_common::span2;
    use analyse::apply;
    use std::cell::{Cell, RefCell};
    use parse::tree::{Char, Expr, NewInstance, NativeChar};

    #[test]
    fn test_simple() {
        let content = r#"
class Native__Char
end

class Char(underlying: Native__Char)
end

def main: Char
  'a'
end
        "#;
        let file = unwrap!(Ok, parse::apply(content.trim(), ""));
        let root = build(&[file.deref()]);

        apply(&[file.deref()], &root);

        assert_eq!(
            unsafe { &*root.find_method("main").unwrap().parse }.exprs.get(0).unwrap(),
            &Expr::Char(Box::new(Char {
                span: span2(8, 3, "'a'", file.deref()),
                instance: RefCell::new(Some(
                    NewInstance {
                        name_opt: None,
                        args: vec![
                            Expr::NewInstance(Box::new(NewInstance {
                                name_opt: None,
                                args: vec![
                                    Expr::NativeChar(Box::new(NativeChar { value: 'a' }))
                                ],
                                def_opt: Cell::new(Some(root.find_class("Native__Char").unwrap().parse))
                            })),
                        ],
                        def_opt: Cell::new(Some(root.find_class("Char").unwrap().parse))
                    }
                ))
            }))
        )
    }
}
