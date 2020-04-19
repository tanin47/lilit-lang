use LilitFile;
use inkwell::module::Module;
use inkwell::context::Context;
use inkwell::builder::Builder;
use parse::tree::{CompilationUnitItem, Class};
use inkwell::values::{IntValue, PointerValue, ArrayValue, BasicValueEnum};
use emit::def::method::EmitterMethod;
use emit::def::class::ClassEmitter;
use inkwell::types::{StructType, BasicTypeEnum};
use inkwell::AddressSpace;

pub mod def;
pub mod expr;
pub mod helper;

struct Emitter<'r> {
    context: Context,
    builder: Builder,
    module: &'r Module,
    va_list_struct_type: StructType,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Value<'def> {
    Void,
    Char(IntValue),
    Int(IntValue),
    String(PointerValue),
    Struct(PointerValue, *const Class<'def>),
    Class(PointerValue, *const Class<'def>),
}

pub fn apply(files: &[&LilitFile]) -> Module {
    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();
    let va_list_struct_type = context.struct_type(
        &vec![
            BasicTypeEnum::IntType(context.i32_type()),
            BasicTypeEnum::IntType(context.i32_type()),
            BasicTypeEnum::PointerType(context.i8_type().ptr_type(AddressSpace::Generic)),
            BasicTypeEnum::PointerType(context.i8_type().ptr_type(AddressSpace::Generic)),
        ],
        false,
    );
    let emitter = Emitter {
        context,
        builder,
        module: &module,
        va_list_struct_type,
    };

    emitter.apply(files);

    module
}

impl <'r> Emitter<'r> {
    fn apply<'def>(&self, files: &[&LilitFile<'def>]) {
        for file in files {
            self.apply_file(file);
        }
    }

    fn apply_file<'def>(&self, file: &LilitFile<'def> ) {
        for item in &file.unit.items {
            match item {
                CompilationUnitItem::Class(class) => self.apply_class_def(class),
                CompilationUnitItem::Method(method) => (),
            }
        }

        for item in &file.unit.items {
            match item {
                CompilationUnitItem::Class(class) => self.apply_class(class),
                CompilationUnitItem::Method(method) => self.apply_method(method),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use index::build;
    use ::{parse, analyse};
    use parse::tree::{CompilationUnit, Type, CompilationUnitItem, Method, Invoke, Expr, Int, NewInstance, NativeInt};
    use test_common::span2;
    use std::cell::{Cell, RefCell};
    use emit::apply;

    #[test]
    fn test_full() {
        let content = r#"
class Native__Void
end

class Void
end

class Native__Int
end

class Int(underlying: Native__Int)
end

class Native__String
end

class String(underlying: Native__String)
end

def native__printf(text: Native__String): Native__Void
end

def println(text: String): Void
  native__printf(text.underlying)
end

def main: Int
  println("Hello world!")
  123
end
        "#;
        let file = unwrap!(Ok, parse::apply(content.trim(), ""));
        let root = build(&[file.deref()]);

        analyse::apply(&[file.deref()], &root);

        let module = apply(&[file.deref()]);
        module.print_to_stderr();
    }
}
