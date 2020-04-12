use emit::{Emitter, Value};
use parse::tree::MemberAccess;
use emit::expr::ExprEmitter;
use inkwell::values::BasicValueEnum;

pub trait MemberAccessEmitter {
    fn apply_member_access<'def>(&self, member_access: &MemberAccess<'def>) -> Value<'def>;
}

impl MemberAccessEmitter for Emitter<'_> {
    fn apply_member_access<'def>(&self, member_access: &MemberAccess<'def>) -> Value<'def> {
        let (parent_ptr, parent_class)  = unwrap2!(Value::Class, self.apply_expr(&member_access.parent));

        let parent_class = unsafe { &*parent_class };
        let param = unsafe { &*member_access.def_opt.get().unwrap() };
        let param_class = unsafe { &*param.tpe.def_opt.get().unwrap() };

        let param_ptr = unsafe {
            self.builder.build_struct_gep(
                parent_ptr,
                param.index as u32,
                format!("Gep field {} of {}", param.index, parent_class.name.fragment).as_ref()
            )
        };

        let value = self.builder.build_load(param_ptr, format!("Load field {} for identifier {}", param.name.fragment, member_access.name.fragment).as_ref());

        Value::Class(unwrap!(BasicValueEnum::PointerValue, value), param_class)
    }
}
