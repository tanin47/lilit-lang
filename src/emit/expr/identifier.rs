use emit::{Emitter, Value};
use parse::tree::{Identifier, IdentifierSource};
use inkwell::values::BasicValueEnum;
use inkwell::types::BasicTypeEnum;
use emit::helper::Helper;

pub trait IdentifierEmitter {
    fn apply_identifier<'def>(&self, identifier: &Identifier<'def>) -> Value<'def>;
}

impl IdentifierEmitter for Emitter<'_> {
    fn apply_identifier<'def>(&self, identifier: &Identifier<'def>) -> Value<'def> {
        match identifier.def_opt.get().unwrap() {
            IdentifierSource::Param(param) => {
                let param = unsafe { &*param };
                let param_class = unsafe { &*param.tpe.def_opt.get().unwrap() };

                let alloca_ptr = param.llvm.get().unwrap();
                self.read_ptr(alloca_ptr, param_class)
            },
            IdentifierSource::Assignment(assignment) => {
                let assignment = unsafe { &*assignment };
                let var_class = unsafe { &*assignment.tpe.get().unwrap() };

                let alloca_ptr = assignment.llvm.get().unwrap();
                self.read_ptr(alloca_ptr, var_class)
            }
        }
    }
}
