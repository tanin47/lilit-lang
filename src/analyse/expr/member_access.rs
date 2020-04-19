use analyse::scope::Scope;
use parse::tree::MemberAccess;
use analyse::expr;
use analyse::tpe::GetType;

pub fn apply<'def>(
    member_access: &mut MemberAccess<'def>,
    scope: &mut Scope<'def>,
) {
    expr::apply(&mut member_access.parent, scope);

    let class = member_access.parent.get_type(scope);

    for param in &class.params {
        if param.name.unwrap().fragment == member_access.name.unwrap().fragment {
            member_access.param_def = Some(param);
        }
    }
}

#[cfg(test)]
mod tests {
    use index;
    use parse;
    use analyse::apply;
    use parse::tree::{Method, Type, Expr, MemberAccess, NewInstance, LiteralString, NativeString};
    use test_common::span2;
    use std::cell::{Cell, RefCell};
    use std::ops::{Deref, DerefMut};

    #[test]
    fn test_simple() {
        let content = r#"
class Native__String
end

class String(underlying: Native__String)
end

class Test(member: String)
end

def main(): Test
  Test("a").member
end
        "#;
        let mut file = unwrap!(Ok, parse::apply(content.trim(), ""));
        let root = index::build(&[file.deref()]);

        apply(&mut [file.deref_mut()], &root);
        assert_eq!(
            root.find_method("main").exprs,
            vec![
                Expr::MemberAccess(Box::new(MemberAccess {
                    parent: Expr::NewInstance(Box::new(NewInstance {
                        name_opt: Some(span2(11, 3, "Test", file.deref())),
                        args: vec![Expr::String(Box::from(LiteralString {
                            span: span2(11, 8, "\"a\"", file.deref()),
                            instance: Some(Box::new(
                                NewInstance {
                                    name_opt: None,
                                    args: vec![
                                        Expr::NewInstance(Box::new(NewInstance {
                                            name_opt: None,
                                            args: vec![
                                                Expr::NativeString(Box::new(NativeString { value: "a".to_string() }))
                                            ],
                                            class_def: Some(root.find_class("Native__String"))
                                        })),
                                    ],
                                    class_def: Some(root.find_class("String"))
                                }
                            ))
                        }))],
                        class_def: Some(root.find_class("Test"))
                    })),
                    name: Some(span2(11, 13, "member", file.deref())),
                    param_def: Some(root.find_class("Test").params.get(0).unwrap())
                }))
            ]
        )
    }
}
