use std::collections::HashMap;
use semantics::tree;
use syntax;
use std::cell::Cell;
use std::io::prelude::*;
use std::rc::Rc;

pub enum ScopeValue {
	Var(*const tree::Var),
	Func(*const tree::Func),
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

	pub fn declare(&mut self, key: &str, value: ScopeValue) {
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
		let value = Box::new(tree::Var {
			llvm_ref: Cell::new(None),
			name: "a".to_string(),
		});

		let mut scope = Scope { levels: Vec::new() };

		scope.enter();
		assert_eq!(scope.levels.len(), 1);
		scope.declare("a", ScopeValue::Var(value.as_ref() as *const tree::Var));
		{
			assert_eq!(scope.read_var("a").unwrap() as *const tree::Var, value.as_ref() as *const tree::Var);
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
			name: "a".to_string(),
            exprs: vec![],
        });

        let mut scope = Scope { levels: Vec::new() };

        scope.enter();
        scope.declare("a", ScopeValue::Var(outer_value.as_ref() as *const tree::Var));
        scope.enter();
        scope.declare("a", ScopeValue::Func(inner_value.as_ref() as *const tree::Func));
        assert_eq!(scope.read_func("a").unwrap() as *const tree::Func, inner_value.as_ref() as *const tree::Func);

        scope.leave();
        assert_eq!(scope.read_var("a").unwrap() as *const tree::Var,outer_value.as_ref() as *const tree::Var);
    }
}
