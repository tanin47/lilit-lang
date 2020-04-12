use emit::{Emitter, Value};
use parse::tree::{Int, NativeInt};

pub trait NativeIntEmitter {
    fn apply_native_int<'def>(&self, int: &NativeInt) -> Value<'def>;
}

impl NativeIntEmitter for Emitter<'_> {
    fn apply_native_int<'def>(&self, int: &NativeInt) -> Value<'def> {
        Value::Int(self.context.i64_type().const_int(int.value as u64, false))
    }
}
