use parse::tree::{Identifier, Param, IdentifierSource, ParamParent, MemberAccess, Expr};
use analyse::scope::Scope;
use std::cell::{Cell, RefCell};

pub fn apply<'def>(
    identifier: &Identifier<'def>,
    scope: &mut Scope<'def>,
) {
    let source = scope.find_identifier(identifier.name.unwrap().fragment).unwrap();

    if let IdentifierSource::Param(param) = source {
        let param = unsafe { &* param };
        if let Some(ParamParent::Class(class)) = param.parent.get() {
            let parent_method = scope.find_parent_method();
            identifier.source.replace(Some(IdentifierSource::ClassParam(Box::new(MemberAccess {
                parent: Expr::Identifier(Box::new(Identifier {
                    name: None,
                    source: RefCell::new(Some(IdentifierSource::Param(parent_method.params.get(0).unwrap())))
                })),
                name: None,
                param_def: Cell::new(Some(param)),
            }))));
            return;
        }
    }

    identifier.source.replace(Some(source));
}

#[cfg(test)]
mod tests {
    use index;
    use parse;
    use analyse::apply;
    use parse::tree::{Method, Type, Expr, MemberAccess, NewInstance, LiteralString, NativeString, Invoke, Identifier, IdentifierSource};
    use test_common::span2;
    use std::cell::{Cell, RefCell};
    use std::ops::{Deref, DerefMut};

    #[test]
    fn test_class_param() {
        let content = r#"
class Int
end

class Test(a: Int)
  def run(): Int
    a
  end
end
        "#;
        let mut file = unwrap!(Ok, parse::apply(content.trim(), ""));
        let root = index::build(&[file.deref()]);

        apply(&mut [file.deref_mut()], &root);

        let test_class = root.find_class("Test");
        let run_method = test_class.find_method("run");
        assert_eq!(
            run_method.exprs,
            vec![
                Expr::Identifier(Box::new(Identifier {
                    name: Some(span2(6, 5, "a", file.deref())),
                    source: RefCell::new(Some(IdentifierSource::ClassParam(Box::new(MemberAccess {
                        parent: Expr::Identifier(Box::new(Identifier {
                            name: None,
                            source: RefCell::new(Some(IdentifierSource::Param(run_method.params.get(0).unwrap())))
                        })),
                        name: None,
                        param_def: Cell::new(Some(test_class.find_param("a")))
                    }))))
                }))
            ]
        )
    }
}
