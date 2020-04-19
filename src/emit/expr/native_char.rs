use emit::{Emitter, Value};
use parse::tree::NativeChar;

pub trait NativeCharEmitter {
    fn apply_native_char<'def>(&self, c: &NativeChar) -> Value<'def>;
}

impl NativeCharEmitter for Emitter<'_> {
    fn apply_native_char<'def>(&self, c: &NativeChar) -> Value<'def> {
        Value::Char(self.context.i8_type().const_int(c.value as u64, false))
    }
}
