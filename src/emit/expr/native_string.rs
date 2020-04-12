use parse::tree::NativeString;
use emit::{Value, Emitter};
use emit::helper::Helper;
use inkwell::AddressSpace;

pub trait NativeStringEmitter {
    fn apply_native_string<'def>(&self, string: &NativeString) -> Value<'def>;
}

impl NativeStringEmitter for Emitter<'_> {
    fn apply_native_string<'def>(&self, string: &NativeString) -> Value<'def> {
        let i8_type = self.context.i8_type();
        let i32_type = self.context.i32_type();

        let array_type = i8_type.array_type((string.value.len() + 1) as u32);
        let string_ptr = self.malloc_array(&array_type);

        for (index, c) in string.value.chars().enumerate() {
            let char_ptr = unsafe {
                self.builder.build_in_bounds_gep(
                    string_ptr,
                    &[i32_type.const_int(0, false), i32_type.const_int(index as u64, false)],
                    format!("Gep char {} of string {}", index, string.value).as_ref())
            };
            self.builder.build_store(char_ptr, i8_type.const_int(c as u64, false));
        }
        // Store string terminating symbol
        let last = unsafe {
            self.builder.build_in_bounds_gep(
                string_ptr,
                &[i32_type.const_int(0, false), i32_type.const_int(string.value.len() as u64, false)],
                format!("Gep the last position for the terminating symbol of string {}", string.value).as_ref())
        };
        self.builder.build_store(last, i8_type.const_int(0, false));

        Value::String(
            self.builder.build_pointer_cast(string_ptr, self.context.i8_type().ptr_type(AddressSpace::Generic), "Cast to eliminate the size info")
        )
    }
}
