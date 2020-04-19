use emit::{Emitter, Value};
use parse::tree::{Identifier, IdentifierSource, Assignment};
use inkwell::values::BasicValueEnum;
use inkwell::types::BasicTypeEnum;
use emit::helper::Helper;
use emit::expr::ExprEmitter;
use inkwell::AddressSpace;

pub trait AssignmentEmitter {
    fn apply_assignment<'def>(&self, assignment: &Assignment<'def>) -> Value<'def>;
}

impl AssignmentEmitter for Emitter<'_> {
    fn apply_assignment<'def>(&self, assignment: &Assignment<'def>) -> Value<'def> {
        let class = unsafe { &*assignment.tpe.get().unwrap() };
        let ptr = self.builder.build_alloca(class.llvm.get().unwrap().ptr_type(AddressSpace::Generic), "alloca assignment");

        assignment.llvm.set(Some(ptr));

        let (value, value_class) = unwrap2!(Value::Class, self.apply_expr(&assignment.expr));

        self.builder.build_store(ptr, value);

        Value::Class(value, value_class)
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
class Native__Any
end

class Native__Int
end

class Int(underlying: Native__Int)
end

class Void
end

def test(): Void
  a = 2
end
        "#;
        let mut file = unwrap!(Ok, parse::apply(content.trim(), ""));
        let root = build(&[file.deref()]);

        analyse::apply(&mut [file.deref_mut()], &root);

        let module = apply(&[file.deref()]);
        module.print_to_stderr();
    }
}
