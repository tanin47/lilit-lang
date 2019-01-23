pub mod gen;

pub mod boolean;
pub mod int8;
pub mod int32;
pub mod string;
pub mod array;
pub mod char;

pub use self::gen::instantiate;
pub use self::gen::gen_invoke;
pub use self::gen::gen_malloc;
pub use self::gen::gen_malloc_array;
pub use self::gen::gen_malloc_dynamic_array;
pub use self::gen::gen_gc_init;
pub use self::gen::gen_gc_collect;
pub use self::gen::get_external_func;

