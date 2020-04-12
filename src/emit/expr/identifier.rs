use emit::{Emitter, Value};
use parse::tree::Identifier;
use inkwell::values::BasicValueEnum;
use inkwell::types::BasicTypeEnum;
use emit::helper::Helper;

pub trait IdentifierEmitter {
    fn apply_identifier<'def>(&self, identifier: &Identifier<'def>) -> Value<'def>;
}

impl IdentifierEmitter for Emitter<'_> {
    fn apply_identifier<'def>(&self, identifier: &Identifier<'def>) -> Value<'def> {
        let param = unsafe { &*identifier.def_opt.get().unwrap() };
        let param_class = unsafe { &*param.tpe.def_opt.get().unwrap() };

        let alloca_ptr = param.llvm.get().unwrap();
        self.read_ptr(alloca_ptr, param_class)
    }
}
