use parse::tree::Invoke;
use emit::{Value, Emitter};
use emit::expr::ExprEmitter;
use inkwell::values::BasicValueEnum;

pub trait InvokeEmitter {
    fn apply_invoke<'def>(&self, invoke: &Invoke<'def>) -> Value<'def>;
}

impl InvokeEmitter for Emitter<'_> {
    fn apply_invoke<'def>(&self, invoke: &Invoke<'def>) -> Value<'def> {
        // TODO: support invoking instance method
        let method = unsafe { &*invoke.def_opt.get().unwrap() };

        let mut args = vec![];

        for (param, arg) in method.params.iter().zip(&invoke.args) {
            let (ptr, _) = unwrap2!(Value::Class, self.apply_expr(arg));
            let param_class = unsafe { &*param.tpe.def_opt.get().unwrap() };

            args.push(BasicValueEnum::PointerValue(ptr));
        }

        let llvm_ret = self.builder.build_call(
            method.llvm.get().unwrap(),
            &args,
            &method.name.fragment);

        let return_type_class = unsafe { &*method.return_type.def_opt.get().unwrap() };

        match return_type_class.name.fragment {
            "Void" | "Native__Void" => Value::Void,
            other => Value::Class(unwrap!(BasicValueEnum::PointerValue, llvm_ret.try_as_basic_value().left().unwrap()), return_type_class),
        }
    }
}
