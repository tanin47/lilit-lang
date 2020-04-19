use parse::tree::{Identifier, Param, Assignment};
use analyse::scope::Scope;
use analyse::expr;
use analyse::tpe::GetType;

pub fn apply<'def>(
    assignment: &Assignment<'def>,
    scope: &mut Scope<'def>,
) {
    expr::apply(&assignment.expr, scope);
    assignment.tpe.set(Some(assignment.expr.get_type(scope)));

    scope.add_var(assignment);
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use index::build;
    use parse;
    use test_common::{span2, make_int_instance};
    use analyse::apply;
    use std::cell::{Cell, RefCell};
    use parse::tree::{Expr, Assignment, Int, NewInstance, Identifier, IdentifierSource};

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
  a = 2
  a
end
        "#;
        let file = unwrap!(Ok, parse::apply(content.trim(), ""));
        let root = build(&[file.deref()]);

        apply(&[file.deref()], &root);

        assert_eq!(
            unsafe { &*root.find_method("main").unwrap().parse }.exprs,
            vec![
                Expr::Assignment(Box::new(Assignment {
                    name: span2(11, 3, "a", file.deref()),
                    expr: Box::new(Expr::Int(Box::new(Int {
                        span: span2(11, 7, "2", file.deref()),
                        instance: RefCell::new(Some(make_int_instance(2, &root)))
                    }))),
                    tpe: Cell::new(Some(root.find_class("Int").unwrap().parse)),
                    llvm: Cell::new(None)
                })),
                Expr::Identifier(Box::new(Identifier {
                    name: span2(12, 3, "a", file.deref()),
                    def_opt: Cell::new(Some(IdentifierSource::Assignment(
                        unwrap!(Expr::Assignment, unsafe { &*root.find_method("main").unwrap().parse }.exprs.get(0).unwrap()).deref()
                    )))
                })),
            ]
        )
    }
}
