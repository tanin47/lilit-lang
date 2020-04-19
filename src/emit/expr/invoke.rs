use parse::tree::Invoke;
use emit::{Value, Emitter};
use emit::expr::ExprEmitter;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;
use emit::helper::Helper;

pub trait InvokeEmitter {
    fn apply_invoke<'def>(&self, invoke: &Invoke<'def>) -> Value<'def>;
    fn apply_native_invoke<'def>(&self, invoke: &Invoke<'def>) -> Value<'def>;
}

impl InvokeEmitter for Emitter<'_> {
    fn apply_invoke<'def>(&self, invoke: &Invoke<'def>) -> Value<'def> {
        if invoke.name.fragment.starts_with("native__") {
            return self.apply_native_invoke(invoke);
        }

        let method = unsafe { &*invoke.def_opt.get().unwrap() };
        let mut args = vec![];

        for (param, arg) in method.params.iter().zip(&invoke.args) {
            let (ptr, arg_class) = unwrap2!(Value::Class, self.apply_expr(arg));
            let param_class = unsafe { &*param.tpe.def_opt.get().unwrap() };
            let arg_class = unsafe { &*arg_class };

            if arg_class.name.fragment == "Native__Null" {
                args.push(BasicValueEnum::PointerValue(param_class.llvm.get().unwrap().ptr_type(AddressSpace::Generic).const_null()));
            } else {
                args.push(BasicValueEnum::PointerValue(ptr));
            }
        }

        let llvm_ret = self.builder.build_call(
            method.llvm.get().unwrap(),
            &args,
            &method.name.fragment);

        let return_type_class = unsafe { &*method.return_type.def_opt.get().unwrap() };

        match return_type_class.name.fragment {
            "Void"  => Value::Void,
            other => Value::Class(unwrap!(BasicValueEnum::PointerValue, llvm_ret.try_as_basic_value().left().unwrap()), return_type_class),
        }
    }

    fn apply_native_invoke<'def>(&self, invoke: &Invoke<'def>) -> Value<'def> {
        let method = unsafe { &*invoke.def_opt.get().unwrap() };
        let mut args = vec![];

        for (index, arg) in invoke.args.iter().enumerate() {
            let param = if index < method.params.len() {
                method.params.get(index).unwrap()
            } else {
                assert!(method.params.last().unwrap().is_varargs, "The last param's varargs isn't true");
                method.params.last().unwrap()
            };
            let (ptr, arg_class) = unwrap2!(Value::Class, self.apply_expr(arg));

            let arg_class = unsafe { &*arg_class };
            assert!(arg_class.name.fragment.starts_with("Native__"), "Expect {} to be a native class", arg_class.name.fragment);

            let param_class = unsafe { &*param.tpe.def_opt.get().unwrap() };
            assert!(param_class.name.fragment.starts_with("Native__"), "Expect {} to be a native class", param_class.name.fragment);

            args.push(
                match arg_class.name.fragment {
                    "Native__Null" => match param_class.name.fragment {
                        "Native__String" => BasicValueEnum::PointerValue(self.context.i8_type().ptr_type(AddressSpace::Generic).const_null()),
                        other => panic!("Null only works with Native__String, not {}", other)
                    },
                    _ => {
                        let first_arg_ptr = unsafe { self.builder.build_struct_gep(ptr, 0, "Gep the native value") };
                        self.builder.build_load(first_arg_ptr, "Load the native value")
                    }
                }
            );
        }

        let llvm_ret = self.builder.build_call(
            method.llvm.get().unwrap(),
            &args,
            &method.name.fragment);

        let return_type_class = unsafe { &*method.return_type.def_opt.get().unwrap() };
        let value = match return_type_class.name.fragment {
            "Native__Int" => Value::Int(unwrap!(BasicValueEnum::IntValue, llvm_ret.try_as_basic_value().left().unwrap())),
            "Native__Char" => Value::Char(unwrap!(BasicValueEnum::IntValue, llvm_ret.try_as_basic_value().left().unwrap())),
            "Native__String" => Value::String(unwrap!(BasicValueEnum::PointerValue, llvm_ret.try_as_basic_value().left().unwrap())),
            "Native__Void" => Value::Void,
            other => panic!("Unsupported {}", other),
        };

        // Lift value to a class
        if Value::Void == value {
            Value::Void
        } else {
            Value::Class(self.convert(&value, return_type_class), return_type_class)
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
class Native__Any
end

class Native__Int
end

class Int(underlying: Native__Int)
end

class Native__String
end

class String(underlying: Native__String)
end

def native__vprintf(s: Native__String, args...: Native__Any): Native__Int
end

class Void
end

def println(text: String, args1: Int, args2: Int): Void
  native__vprintf(text.underlying, args1.underlying, args2.underlying)
end

def test(): Void
  println("Test %d %d", 1, 2)
end
        "#;
        let file = unwrap!(Ok, parse::apply(content.trim(), ""));
        let root = build(&[file.deref()]);

        analyse::apply(&[file.deref()], &root);

        let module = apply(&[file.deref()]);
        module.print_to_stderr();
    }
}
