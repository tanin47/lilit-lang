use parse::tree::Class;
use emit::Emitter;
use inkwell::AddressSpace;
use inkwell::types::BasicTypeEnum;
use emit::helper::Helper;

pub trait ClassEmitter {
    fn apply_class_def(&self, class: &Class);
    fn apply_class(&self, class: &Class);
    fn get_type_enums_for_class(&self, class: &Class) -> Vec<BasicTypeEnum>;
    fn get_type_enums_for_native(&self, class: &Class) -> Vec<BasicTypeEnum>;
}

impl ClassEmitter for Emitter<'_> {
    fn apply_class_def(&self, class: &Class) {
        let opaque_struct = self.context.opaque_struct_type(class.name.fragment);

        if class.name.fragment.starts_with("Native__Struct__") {
            class.llvm.set(Some(opaque_struct));
            let native_opaque_struct = self.context.opaque_struct_type(class.name.fragment);
            class.llvm_native.set(Some(native_opaque_struct));
        } else if class.name.fragment.starts_with("Native__") {
            class.llvm.set(Some(opaque_struct));
            class.llvm_native.set(Some(opaque_struct));
        } else {
            class.llvm.set(Some(opaque_struct));
        }
    }

    fn apply_class(&self, class: &Class) {
        // TODO: this isn't a good representation. Refactor it somehow.
        // There are 2 concepts: 1. normal class, 2. native class, and 3. native struct class.
        if class.name.fragment.starts_with("Native__Struct__") {
            class.llvm.get().unwrap().set_body(&self.get_type_enums_for_class(class), false);
            class.llvm_native.get().unwrap().set_body(&self.get_type_enums_for_native(class), false);
        } else if class.name.fragment.starts_with("Native__") {
            class.llvm.get().unwrap().set_body(&self.get_type_enums_for_native(class), false);
        } else {
            class.llvm.get().unwrap().set_body(&self.get_type_enums_for_class(class), false);
        }
    }

    fn get_type_enums_for_class(&self, class: &Class) -> Vec<BasicTypeEnum> {
        let mut type_enums = vec![];
        for param in &class.params {
            let param_class = unsafe { &* param.tpe.def_opt.get().unwrap() };
            type_enums.push(param_class.llvm.get().unwrap().ptr_type(AddressSpace::Generic).into());
        }
        type_enums
    }

    fn get_type_enums_for_native(&self, class: &Class) -> Vec<BasicTypeEnum> {
       match class.name.fragment {
           "Native__Char" => vec![self.context.i8_type().into()],
           "Native__Int" => vec![self.context.i64_type().into()],
           "Native__String" => vec![self.context.i8_type().ptr_type(AddressSpace::Generic).into()],
           "Native__Void" => vec![],
           "Native__Any" => vec![],
           "Native__Null" => vec![],
           other if other.starts_with("Native__Struct__") => {
               let mut params = vec![];
               for param in &class.params {
                   params.push(self.get_type_for_native(unsafe { &*param.tpe.def_opt.get().unwrap() }));
               }
               params
           },
           other => panic!("Unknown native class: {}", other),
       }
    }
}
