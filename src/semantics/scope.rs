use std::collections::HashMap;
use semantics::tree;
use syntax;
use std::cell::Cell;
use std::io::prelude::*;
use std::rc::Rc;

pub enum ScopeValue {
	Var(*const tree::Var),
	Member(*const tree::Var, *const tree::Var),
	Func(*const tree::Func),
	Method(*const tree::Func),
	StaticMethod(*const tree::Func),
	Class(*const tree::Class),
}

pub struct Scope {
	pub levels: Vec<HashMap<String, ScopeValue>>,
}

impl Scope {
	pub fn enter(&mut self) {
		self.levels.push(HashMap::new());
	}

	pub fn leave(&mut self) {
		self.levels.pop();
	}

	pub fn make_method_key(class_name: &str, method_name: &str) -> String {
		format!("__{}__{}", class_name, method_name)
	}

	pub fn make_static_method_key(class_name: &str, method_name: &str) -> String {
		format!("__static__{}__{}", class_name, method_name)
	}

	pub fn declare(&mut self, value: ScopeValue) {
		let key = match value {
			ScopeValue::Var(var_ptr) => (unsafe { &*var_ptr }).name.to_string(),
			ScopeValue::Member(var_ptr, this) => (unsafe { &*var_ptr }).name.to_string(),
			ScopeValue::Func(func_ptr) => (unsafe { &*func_ptr }).name.to_string(),
			ScopeValue::Class(class_ptr) => (unsafe { &*class_ptr }).name.to_string(),
			ScopeValue::Method(ref func_ptr) => {
				let func = unsafe { &**func_ptr };
				let func_name = &func.name;
				let klass = unsafe { &*func.parent_class_opt.get().unwrap() };

                Scope::make_method_key(&klass.name, func_name)
			},
			ScopeValue::StaticMethod(ref func_ptr) => {
				let func = unsafe { &**func_ptr };
				let func_name = &func.name;
				let klass = unsafe { &*func.parent_class_opt.get().unwrap() };

				Scope::make_static_method_key(&klass.name, func_name)
			},
		};
		let last_index = self.levels.len() - 1;
		let map = &mut self.levels[last_index];
		map.insert(key.to_string(), value);
	}

	pub fn read(&self, key: &str) -> Option<&ScopeValue> {
		for i in (0..self.levels.len()).rev() {
            let map = &self.levels[i];
            match map.get(key) {
                Some(value) => return Some(value),
                None => (),
            }
		}

        None
	}

	pub fn read_class(&self, class_name: &str) -> Option<&tree::Class> {
		match self.read(class_name) {
			Some(&ScopeValue::Class(class)) => Some(unsafe { &*class }),
			_ => None
		}
	}

	pub fn read_method(&self, class_name: &str, method_name: &str) -> Option<&tree::Func> {
		match self.read(&Scope::make_method_key(class_name, method_name)) {
			Some(&ScopeValue::Method(func)) => Some(unsafe { &*func }),
			_ => None
		}
	}

	pub fn read_static_method(&self, class_name: &str, method_name: &str) -> Option<&tree::Func> {
		match self.read(&Scope::make_static_method_key(class_name, method_name)) {
			Some(&ScopeValue::StaticMethod(func)) => Some(unsafe { &*func }),
			_ => None
		}
	}

	pub fn read_var(&self, key: &str) -> Option<&tree::Var> {
		match self.read(key) {
			Some(&ScopeValue::Var(var)) => Some(unsafe { &*var }),
			_ => None
		}
	}

	pub fn read_func(&self, key: &str) -> Option<&tree::Func> {
		match self.read(key) {
			Some(&ScopeValue::Func(func)) => Some(unsafe { &*func }),
			_ => None
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_enters_new_scope_declare_var_and_leave() {
		let klass = Box::new(tree::Class {
			name: "klass".to_string(),
		    params: vec![],
		    extends: vec![],
		    methods: vec![],
		});
		let func = Box::new(tree::Func {
			llvm_ref: Cell::new(None),
		    parent_class_opt: Cell::new(Some(klass.as_ref())),
		    name: "run".to_string(),
		    args: vec![],
		    return_type: "Int".to_string(),
		    exprs: vec![],
		});

		let mut scope = Scope { levels: Vec::new() };

		scope.enter();
		assert_eq!(scope.levels.len(), 1);
		scope.declare(ScopeValue::Method(func.as_ref()));
		{
			assert_eq!(scope.read_method(&klass.name, &func.name).unwrap() as *const tree::Func, func.as_ref() as *const tree::Func);
		}

		scope.leave();
		assert_eq!(scope.levels.len(), 0);
	}

    #[test]
    fn it_enters_two_level_and_use_the_right_var() {
        let outer_value = Box::new(tree::Var {
            llvm_ref: Cell::new(None),
            name: "a".to_string(),
        });
        let inner_value = Box::new(tree::Func {
            llvm_ref: Cell::new(None),
            parent_class_opt: Cell::new(None),
			name: "a".to_string(),
			args: vec![],
			return_type: "Int".to_string(),
            exprs: vec![],
        });

        let mut scope = Scope { levels: Vec::new() };

        scope.enter();
        scope.declare(ScopeValue::Var(outer_value.as_ref() as *const tree::Var));
        scope.enter();
        scope.declare(ScopeValue::Func(inner_value.as_ref() as *const tree::Func));
        assert_eq!(scope.read_func("a").unwrap() as *const tree::Func, inner_value.as_ref() as *const tree::Func);

        scope.leave();
        assert_eq!(scope.read_var("a").unwrap() as *const tree::Var,outer_value.as_ref() as *const tree::Var);
    }
}
