use emit::{Emitter, Value};
use parse::tree::{Identifier, IdentifierSource};
use inkwell::values::BasicValueEnum;
use inkwell::types::BasicTypeEnum;
use emit::helper::Helper;
use emit::expr::member_access::MemberAccessEmitter;

pub trait IdentifierEmitter {
    fn apply_identifier<'def>(&self, identifier: &Identifier<'def>) -> Value<'def>;
}

impl IdentifierEmitter for Emitter<'_> {
    fn apply_identifier<'def>(&self, identifier: &Identifier<'def>) -> Value<'def> {
        match identifier.source.borrow().as_ref().unwrap() {
            IdentifierSource::Param(param) => {
                let param = unsafe { &**param };
                let param_class = unsafe { &*param.tpe.class_def.get().unwrap() };

                let alloca_ptr = param.llvm.get().unwrap();
                self.read_ptr(alloca_ptr, param_class)
            },
            IdentifierSource::Assignment(assignment) => {
                let assignment = unsafe { &**assignment };
                let var_class = unsafe { &*assignment.tpe.get().unwrap() };

                let alloca_ptr = assignment.llvm.get().unwrap();
                self.read_ptr(alloca_ptr, var_class)
            },
            IdentifierSource::ClassParam(c) => {
                self.apply_member_access(c)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use analyse;
    use index;
    use emit;
    use parse;
    use std::ops::{Deref, DerefMut};

    #[test]
    fn test_simple() {
        let content = r#"
class Int
end

class Test(a: Int)
  def test(): Int
    a
  end
end
        "#;
        let mut file = unwrap!(Ok, parse::apply(content.trim(), ""));
        let root = index::build(&[file.deref()]);

        analyse::apply(&mut [file.deref_mut()], &root);

        let module = emit::apply(&[file.deref()]);
        module.print_to_stderr();
    }
}
