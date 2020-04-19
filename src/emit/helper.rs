use inkwell::types::{StructType, FunctionType, ArrayType};
use inkwell::values::{PointerValue, FunctionValue, BasicValueEnum};
use emit::{Emitter, Value};
use inkwell::AddressSpace;
use inkwell::attributes::Attribute;
use inkwell::module::Linkage;
use parse::tree::Class;
use emit::expr::new_instance::NewInstanceEmitter;

pub trait Helper {
    fn malloc_array(&self, array_type: &ArrayType) -> PointerValue;
    fn malloc(&self, struct_type: &StructType) -> PointerValue;
    fn get_external_func(&self, name: &str, tpe: FunctionType) -> FunctionValue;
    fn convert<'def>(&self, value: &Value<'def>, expected_class: &Class<'def>) -> PointerValue;
    fn gc_init(&self);
    fn gc_collect(&self);
    fn gc_register_finalizer(&self, ptr: PointerValue);
    fn read_ptr<'def>(&self, alloca_ptr: PointerValue, class: &Class<'def>) -> Value<'def>;
}

impl Helper for Emitter<'_> {
    fn malloc_array(&self, array_type: &ArrayType) -> PointerValue {
        let func_type = self.context
            .i8_type().ptr_type(AddressSpace::Generic)
            .fn_type(&[self.context.i64_type().into()], false);
        let func = self.get_external_func("GC_malloc", func_type);
        func.add_attribute(0, self.context.create_enum_attribute(Attribute::get_named_enum_kind_id("noalias"), 0));

        let p = match self.builder.build_call(func, &[array_type.size_of().unwrap().into()], "malloc").try_as_basic_value().left().unwrap() {
            BasicValueEnum::PointerValue(p) => p,
            other => panic!("Expect BasicValueEnum::PointerValue, found {:?}", other),
        };
        self.gc_register_finalizer(p);

        self.builder.build_pointer_cast(p, array_type.ptr_type(AddressSpace::Generic), "Cast pointer to ArrayType")
    }

    fn malloc(&self, struct_type: &StructType) -> PointerValue {
        let func_type = self.context
            .i8_type().ptr_type(AddressSpace::Generic)
            .fn_type(&[self.context.i64_type().into()], false);
        let func = self.get_external_func("GC_malloc", func_type);
        func.add_attribute(0, self.context.create_enum_attribute(Attribute::get_named_enum_kind_id("noalias"), 0));

        let cast_size = self.builder.build_int_cast(struct_type.size_of().unwrap(), self.context.i64_type(), "cast_size");
        let p = match self.builder.build_call(func, &[cast_size.into()], "malloc").try_as_basic_value().left().unwrap() {
            BasicValueEnum::PointerValue(p) => p,
            x => panic!("Expect BasicValueEnum::PointerValue, found {:?}", x),
        };
        self.gc_register_finalizer(p);

        self.builder.build_pointer_cast(p, struct_type.ptr_type(AddressSpace::Generic), "cast")
    }

    fn get_external_func(
        &self,
        name: &str,
        tpe: FunctionType,
    ) -> FunctionValue {
        match self.module.get_function(name) {
            Some(f) => f,
            None => self.module.add_function(name, tpe, Some(Linkage::External)),
        }
    }

    fn convert<'def>(&self, value: &Value<'def>, expected_class: &Class<'def>) -> PointerValue {
        match value {
            Value::Char(i) => {
                assert_eq!("Native__Char", expected_class.name.fragment);
                let instance = self.malloc(&expected_class.llvm.get().unwrap());

                let param_ptr = unsafe {
                    self.builder.build_struct_gep(instance, 0 as u32, format!("Gep for the native param of the class {}", expected_class.name.fragment).as_ref())
                };
                self.builder.build_store(param_ptr, BasicValueEnum::IntValue(*i));
                instance
            },
            Value::Int(i) => {
                assert_eq!("Native__Int", expected_class.name.fragment);
                let instance = self.malloc(&expected_class.llvm.get().unwrap());

                let param_ptr = unsafe {
                    self.builder.build_struct_gep(instance, 0 as u32, format!("Gep for the native param of the class {}", expected_class.name.fragment).as_ref())
                };
                self.builder.build_store(param_ptr, BasicValueEnum::IntValue(*i));
                instance
            },
            Value::String(i) => {
                assert_eq!("Native__String", expected_class.name.fragment);
                let instance = self.malloc(&expected_class.llvm.get().unwrap());

                let param_ptr = unsafe {
                    self.builder.build_struct_gep(instance, 0 as u32, format!("Gep for the native param of the class {}", expected_class.name.fragment).as_ref())
                };
                self.builder.build_store(param_ptr, BasicValueEnum::PointerValue(*i));
               instance
            },
            Value::Class(ptr, class) => {
                let class = unsafe { &**class };
                assert_eq!(expected_class.name.fragment, class.name.fragment);
                *ptr
            },
            Value::Void => panic!(),
        }
    }

    fn gc_init(&self) {
        let fn_type = self.context
            .void_type()
            .fn_type(&[], false);
        let func = self.get_external_func("GC_init", fn_type);

        self.builder.build_call(func, &[], "gc_init");
    }

    fn gc_collect(&self) {
        let fn_type = self.context
            .void_type()
            .fn_type(&[], false);
        let func = self.get_external_func("GC_gcollect", fn_type);

        self.builder.build_call(func, &[], "gc_gcollect");
    }

    fn gc_register_finalizer(&self, ptr: PointerValue) {
        let finalizer_func = self.get_external_func(
            "GC_finalizer",
            self.context.void_type().fn_type(
                &[
                    self.context.i8_type().ptr_type(AddressSpace::Generic).into(),
                    self.context.i8_type().ptr_type(AddressSpace::Generic).into()
                ],
                false
            ),
        );

        let param_types = vec![
            self.context.i8_type().ptr_type(AddressSpace::Generic).into(),
            finalizer_func.as_global_value().as_pointer_value().get_type().into(),
            self.context.i8_type().ptr_type(AddressSpace::Generic).into(),
            finalizer_func.get_type().ptr_type(AddressSpace::Generic).into(),
            self.context.i8_type().ptr_type(AddressSpace::Generic).ptr_type(AddressSpace::Generic).into(),
        ];
        let func_type = self.context
            .void_type()
            .fn_type(&param_types, false);
        let func = self.get_external_func("GC_register_finalizer", func_type);

        self.builder.build_call(
            func,
            &[
                ptr.into(),
                finalizer_func.as_global_value().as_pointer_value().into(),
                self.context.i8_type().ptr_type(AddressSpace::Generic).const_null().into(),
                finalizer_func.get_type().ptr_type(AddressSpace::Generic).const_null().into(),
                self.context.i8_type().ptr_type(AddressSpace::Generic).ptr_type(AddressSpace::Generic).const_null().into(),
            ],
            "register_finalizer"
        );
    }

    fn read_ptr<'def>(&self, alloca_ptr: PointerValue, class: &Class<'def>) -> Value<'def> {
        let value = self.builder.build_load(
            alloca_ptr,
            format!("Read ptr into {}", class.name.fragment).as_ref()
        );

        Value::Class(unwrap!(BasicValueEnum::PointerValue, value), class)
    }
}
