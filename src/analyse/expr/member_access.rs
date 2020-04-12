use analyse::scope::Scope;
use parse::tree::MemberAccess;
use analyse::expr;
use analyse::tpe::GetType;

pub fn apply<'def>(
    member_access: &MemberAccess<'def>,
    scope: &mut Scope<'def>,
) {
    expr::apply(&member_access.parent, scope);

    let class = member_access.parent.get_type(scope);

    for param in &class.params {
        if param.name.fragment == member_access.name.fragment {
            member_access.def_opt.set(Some(param));
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
    use std::ops::Deref;

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
        let file = unwrap!(Ok, parse::apply(content.trim(), ""));
        let root = index::build(&[file.deref()]);

        apply(&[file.deref()], &root);
        assert_eq!(
            unsafe { &*root.find_method("main").unwrap().parse }.exprs,
            vec![
                Expr::MemberAccess(Box::new(MemberAccess {
                    parent: Expr::NewInstance(Box::new(NewInstance {
                        name_opt: Some(span2(11, 3, "Test", file.deref())),
                        args: vec![Expr::String(Box::from(LiteralString {
                            span: span2(11, 8, "\"a\"", file.deref()),
                            instance: RefCell::new(Some(
                                NewInstance {
                                    name_opt: None,
                                    args: vec![
                                        Expr::NewInstance(Box::new(NewInstance {
                                            name_opt: None,
                                            args: vec![
                                                Expr::NativeString(Box::new(NativeString { value: "a".to_string() }))
                                            ],
                                            def_opt: Cell::new(Some(root.find_class("Native__String").unwrap().parse))
                                        })),
                                    ],
                                    def_opt: Cell::new(Some(root.find_class("String").unwrap().parse))
                                }
                            ))
                        }))],
                        def_opt: Cell::new(Some(root.find_class("Test").unwrap().parse))
                    })),
                    name: span2(11, 13, "member", file.deref()),
                    def_opt: Cell::new(Some(
                        unsafe { &*root.find_class("Test").unwrap().parse }.params.get(0).unwrap()
                    ))
                }))
            ]
        )
    }
}
