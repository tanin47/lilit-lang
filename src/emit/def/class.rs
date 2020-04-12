use parse::tree::Class;
use emit::Emitter;
use inkwell::AddressSpace;
use inkwell::types::BasicTypeEnum;

pub trait ClassEmitter {
    fn apply_class_def(&self, class: &Class);
    fn apply_class(&self, class: &Class);
    fn get_type_enums_for_class(&self, class: &Class) -> Vec<BasicTypeEnum>;
    fn get_type_enums_for_native(&self, name: &str) -> Vec<BasicTypeEnum>;
}

impl ClassEmitter for Emitter<'_> {
    fn apply_class_def(&self, class: &Class) {
        let opaque_struct = self.context.opaque_struct_type(class.name.fragment);
        class.llvm.set(Some(opaque_struct));
    }

    fn apply_class(&self, class: &Class) {
        let type_enums = if class.name.fragment.starts_with("Native__") {
            self.get_type_enums_for_native(class.name.fragment)
        } else {
            self.get_type_enums_for_class(class)
        };

        class.llvm.get().unwrap().set_body(&type_enums, false);
    }

    fn get_type_enums_for_class(&self, class: &Class) -> Vec<BasicTypeEnum> {
        let mut type_enums = vec![];
        for param in &class.params {
            let param_class = unsafe { &* param.tpe.def_opt.get().unwrap() };
            type_enums.push(param_class.llvm.get().unwrap().ptr_type(AddressSpace::Generic).into());
        }
        type_enums
    }

    fn get_type_enums_for_native(&self, name: &str) -> Vec<BasicTypeEnum> {
       match name {
           "Native__Int" => vec![self.context.i64_type().into()],
           "Native__String" => vec![self.context.i8_type().ptr_type(AddressSpace::Generic).into()],
           "Native__Void" => vec![],
           _ => panic!("Unknown native class: {}", name),
       }
    }
}
