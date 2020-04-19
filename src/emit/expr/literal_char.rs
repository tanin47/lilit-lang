use emit::{Value, Emitter};
use emit::expr::new_instance::NewInstanceEmitter;
use parse::tree::Char;

pub trait LiteralCharEmitter {
    fn apply_literal_char<'def>(&self, int: &Char<'def>) -> Value<'def>;
}

impl LiteralCharEmitter for Emitter<'_> {
    fn apply_literal_char<'def>(&self, int: &Char<'def>) -> Value<'def> {
        self.apply_new_instance(int.instance.as_ref().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::ops::{Deref, DerefMut};

    use index::build;
    use ::{parse, analyse};
    use parse::tree::{CompilationUnit, Type, CompilationUnitItem, Method, Invoke, Expr, Int, NewInstance, NativeInt};
    use test_common::span2;
    use std::cell::{Cell, RefCell};
    use emit::apply;

    #[test]
    fn test_full() {
        let content = r#"
class Native__Char
end

class Char(underlying: Native__Char)
end

def test(): Char
  'a'
end
        "#;
        let mut file = unwrap!(Ok, parse::apply(content.trim(), ""));
        let root = build(&[file.deref()]);

        analyse::apply(&mut [file.deref_mut()], &root);

        let module = apply(&[file.deref()]);
        module.print_to_stderr();
    }
}
