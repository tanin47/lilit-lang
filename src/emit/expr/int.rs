use emit::{Emitter, Value};
use parse::tree::Int;
use emit::expr::new_instance::NewInstanceEmitter;

pub trait IntEmitter {
    fn apply_int<'def>(&self, int: &Int<'def>) -> Value<'def>;
}

impl IntEmitter for Emitter<'_> {
    fn apply_int<'def>(&self, int: &Int<'def>) -> Value<'def> {
        self.apply_new_instance(int.instance.borrow().as_ref().unwrap())
    }
}
