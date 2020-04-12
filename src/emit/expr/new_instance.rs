use parse::tree::NewInstance;
use emit::{Value, Emitter};
use emit::expr::ExprEmitter;
use emit::helper::Helper;

pub trait NewInstanceEmitter {
    fn apply_new_instance<'def>(&self, new_instance: &NewInstance<'def>) -> Value<'def>;
}

impl NewInstanceEmitter for Emitter<'_> {
    fn apply_new_instance<'def>(&self, new_instance: &NewInstance<'def>) -> Value<'def> {
        let class = unsafe { &*new_instance.def_opt.get().unwrap() };

        let instance = self.malloc(&class.llvm.get().unwrap());

        for (index, arg) in new_instance.args.iter().enumerate() {
            let value = self.apply_expr(arg);

            let param_ptr = unsafe {
                self.builder.build_struct_gep(instance, index as u32, format!("Gep for the field #{} of the class {}", index, class.name.fragment).as_ref())
            };

            self.builder.build_store(param_ptr, self.convert(&value));
        }

        Value::Class(instance, new_instance.def_opt.get().unwrap())
    }
}
