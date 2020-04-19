use parse::tree::{Expr, Invoke, Class, Method};
use analyse::scope::Scope;
use analyse::expr;
use analyse::tpe::GetType;

pub fn apply<'def>(
    invoke: &Invoke<'def>,
    scope: &mut Scope<'def>,
) {
    for arg in &invoke.args {
        expr::apply(arg, scope);
    }

    invoke.def_opt.set(Some(
        match &invoke.invoker_opt {
            Some(parent) => {
                expr::apply(parent, scope);
                parent.get_type(scope).find_method(invoke.name.fragment)
            },
            None => scope.find_method(invoke.name.fragment).unwrap().parse,
        }
    ));
}

#[cfg(test)]
mod tests {
    use index;
    use parse;
    use analyse::apply;
    use parse::tree::{Method, Type, Expr, MemberAccess, NewInstance, LiteralString, NativeString, Invoke};
    use test_common::span2;
    use std::cell::{Cell, RefCell};
    use std::ops::{Deref, DerefMut};

    #[test]
    fn test_instance_method() {
        let content = r#"
class Test
  def run(): Test
  end
end

def main(): Test
  Test().run()
end
        "#;
        let mut file = unwrap!(Ok, parse::apply(content.trim(), ""));
        let root = index::build(&[file.deref()]);

        apply(&mut [file.deref_mut()], &root);
        assert_eq!(
            unsafe { &*root.find_method("main").unwrap().parse }.exprs,
            vec![
                Expr::Invoke(Box::new(Invoke {
                    invoker_opt: Some(Expr::NewInstance(Box::new(NewInstance {
                        name_opt: Some(span2(7, 3, "Test", file.deref())),
                        args: vec![],
                        def_opt: Cell::new(Some(root.find_class("Test").unwrap().parse))
                    }))),
                    name: span2(7, 10, "run", file.deref()),
                    args: vec![],
                    def_opt: Cell::new(Some(unsafe { &*root.find_class("Test").unwrap().parse }.find_method("run")))
                }))
            ]
        )
    }
}
