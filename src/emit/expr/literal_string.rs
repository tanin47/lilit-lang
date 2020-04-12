use emit::{Value, Emitter};
use parse::tree::LiteralString;
use emit::expr::new_instance::NewInstanceEmitter;

pub trait LiteralStringEmitter {
    fn apply_literal_string<'def>(&self, int: &LiteralString<'def>) -> Value<'def>;
}

impl LiteralStringEmitter for Emitter<'_> {
    fn apply_literal_string<'def>(&self, int: &LiteralString<'def>) -> Value<'def> {
        self.apply_new_instance(int.instance.borrow().as_ref().unwrap())
    }
}
