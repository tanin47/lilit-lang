use std::collections::HashMap;
use semantics;
use ast;
use std::cell::Cell;
use std::io::prelude::*;

pub struct Scope<'a> {
	pub levels: Vec<HashMap<&'a str, &'a Box<semantics::Var<'a>>>>,
}

impl<'a> Scope<'a> {
	pub fn enter(&mut self) {
		self.levels.push(HashMap::new());
	}

	pub fn leave(&mut self) {
		self.levels.pop();
	}

	pub fn declare(&mut self, key: &'a str, value: &'a Box<semantics::Var<'a>>) {
		let last_index = self.levels.len() - 1;
		let map = &mut self.levels[last_index];
		map.insert(key, value);
	}

	pub fn read(&self, key: &str) -> Option<&&Box<semantics::Var<'a>>> {
		for i in (0..self.levels.len()).rev() {
            let map = &self.levels[i];
            match map.get(key) {
                Some(value) => return Some(value),
                None => (),
            }
		}

        None
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_enters_new_scope_declare_var_and_leave() {
		let var = Box::new(ast::Var { id: Box::new(ast::Id { name: "a".to_string() }) });
		let value = Box::new(semantics::Var {
			llvm_ref: Cell::new(None),
			id: Box::new(semantics::Id { syntax: &var.id } ),
			syntax: &var,
		});

		let mut scope = Scope { levels: Vec::new() };

		scope.enter();
		assert_eq!(scope.levels.len(), 1);
		scope.declare("a", &value);
		assert_eq!((*scope.read("a").unwrap()).as_ref() as *const _, value.as_ref() as *const _);

		scope.leave();
		assert_eq!(scope.levels.len(), 0);
	}

    #[test]
    fn it_enters_two_level_and_use_the_right_var() {
        let var = Box::new(ast::Var { id: Box::new(ast::Id { name: "a".to_string() }) });
        let outer_value = Box::new(semantics::Var {
            llvm_ref: Cell::new(None),
            id: Box::new(semantics::Id { syntax: &var.id } ),
            syntax: &var,
        });
        let inner_value = Box::new(semantics::Var {
            llvm_ref: Cell::new(None),
            id: Box::new(semantics::Id { syntax: &var.id } ),
            syntax: &var,
        });

        let mut scope = Scope { levels: Vec::new() };

        scope.enter();
        scope.declare("a", &outer_value);
        scope.enter();
        scope.declare("a", &inner_value);
        assert_eq!((*scope.read("a").unwrap()).as_ref() as *const _, inner_value.as_ref() as *const _);

        scope.leave();
        assert_eq!((*scope.read("a").unwrap()).as_ref() as *const _, outer_value.as_ref() as *const _);
    }
}
