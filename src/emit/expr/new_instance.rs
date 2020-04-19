use parse::tree::{NewInstance, Class};
use emit::{Value, Emitter};
use emit::expr::ExprEmitter;
use emit::helper::Helper;
use inkwell::values::{PointerValue, BasicValueEnum};

pub trait NewInstanceEmitter {
    fn apply_new_instance<'def>(&self, new_instance: &NewInstance<'def>) -> Value<'def>;
    fn alloc_new_instance<'def>(&self, class: &Class<'def>, args: Vec<Value<'def>>) -> PointerValue;
}

impl NewInstanceEmitter for Emitter<'_> {
    fn apply_new_instance<'def>(&self, new_instance: &NewInstance<'def>) -> Value<'def> {
        let mut args = vec![];
        for  arg in &new_instance.args {
            args.push(self.apply_expr(arg));
        }

        let class = unsafe { &*new_instance.def_opt.get().unwrap() };
        Value::Class(self.alloc_new_instance(class, args), class)
    }

    fn alloc_new_instance<'def>(&self, class: &Class<'def>, args: Vec<Value<'def>>) -> PointerValue {
        let instance = self.malloc(&class.llvm.get().unwrap());

        if class.name.fragment == "Native__Int" || class.name.fragment == "Native__String" || class.name.fragment == "Native__Char" {
            let native_value_ptr = unsafe {
                self.builder.build_struct_gep(instance, 0, format!("Gep for the native value of the class {}", class.name.fragment).as_ref())
            };

            self.builder.build_store(
                native_value_ptr,
                match args.get(0).unwrap() {
                    Value::Char(i) => BasicValueEnum::IntValue(*i),
                    Value::Int(i) => BasicValueEnum::IntValue(*i),
                    Value::String(i) => BasicValueEnum::PointerValue(*i),
                    other => panic!(),
                }
            );
        } else {
            for (index, (param, arg)) in class.params.iter().zip(args.iter()).enumerate() {
                let expected_value_class = unsafe { &*param.tpe.def_opt.get().unwrap() };

                let param_ptr = unsafe {
                    self.builder.build_struct_gep(instance, index as u32, format!("Gep for the field #{} of the class {}", index, class.name.fragment).as_ref())
                };

                self.builder.build_store(param_ptr, self.convert(&arg, expected_value_class));
            }
        }


        instance
    }
}
