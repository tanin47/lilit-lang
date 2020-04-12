use parse::tree::{CompilationUnit, Class, Method};
use index::tree::{Root, RootItem};
use ::{index, parse};

#[derive(Debug, PartialEq, Clone)]
pub struct Scope<'def> {
    pub levels: Vec<Level<'def>>,
}

impl <'def> Scope<'def> {
    pub fn enter(&mut self) {
        self.levels.push(Level { enclosing_opt: None });
    }

    pub fn enter_root(&mut self, root: &Root<'def>) {
        self.levels.push(Level { enclosing_opt: Some(LevelEnclosing::Root(root)) });
    }

    pub fn enter_class(&mut self, class: &Class<'def>) {
        for i in (0..self.levels.len()).rev() {
            let level = self.levels.get(i).unwrap();
            match level.enclosing_opt {
                Some(LevelEnclosing::Root(root)) => {
                    let root = unsafe { &*root };
                    for item in &root.items {
                        if let RootItem::Class(candidate) = item {
                            if candidate.parse == class {
                                self.levels.push(Level { enclosing_opt: Some(LevelEnclosing::Class(candidate)) });
                                return;
                            }
                        }
                    }
                },
                _ => (),
            }
        }

        panic!();
    }

    pub fn enter_method(&mut self, method: &Method<'def>) {
        for i in (0..self.levels.len()).rev() {
            let level = self.levels.get(i).unwrap();
            match level.enclosing_opt {
                Some(LevelEnclosing::Root(root)) => {
                    let root = unsafe { &*root };
                    for item in &root.items {
                        if let RootItem::Method(candidate) = item {
                            if candidate.parse == method {
                                self.levels.push(Level { enclosing_opt: Some(LevelEnclosing::Method(candidate)) });
                                return;
                            }
                        }
                    }
                },
                Some(LevelEnclosing::Class(class)) => {
                    let class = unsafe { &*class };
                    for candidate in &class.methods {
                       if candidate.parse == method  {
                           self.levels.push(Level { enclosing_opt: Some(LevelEnclosing::Method(candidate)) });
                           return;
                       }
                    }
                },
                _ => (),
            }
        }

        panic!();
    }

    pub fn leave(&mut self) {
        self.levels.pop();
    }

    pub fn find_method(&self, name: &str) -> Option<&index::tree::Method<'def>> {
        for i in (0..self.levels.len()).rev() {
            let level = self.levels.get(i).unwrap();
            match level.enclosing_opt {
                Some(LevelEnclosing::Root(root)) => {
                    let root = unsafe { &*root };
                    for item in &root.items {
                        if let RootItem::Method(candidate) = item {
                            if unsafe { &*candidate.parse }.name.fragment == name {
                                return Some(candidate);
                            }
                        }
                    }
                },
                Some(LevelEnclosing::Class(class)) => {
                    let class = unsafe { &*class };
                    for candidate in &class.methods {
                        if unsafe { &*candidate.parse }.name.fragment == name  {
                            return Some(candidate);
                        }
                    }
                },
                _ => (),
            }
        }

        panic!();
    }

    pub fn find_class(&self, name: &str) -> Option<&index::tree::Class<'def>> {
        for i in (0..self.levels.len()).rev() {
            let level = self.levels.get(i).unwrap();
            match level.enclosing_opt {
                Some(LevelEnclosing::Root(root)) => {
                    let root = unsafe { &*root };
                    for item in &root.items {
                        if let RootItem::Class(candidate) = item {
                            if unsafe { &*candidate.parse }.name.fragment == name {
                                return Some(candidate);
                            }
                        }
                    }
                },
                _ => (),
            }
        }

        panic!("Unable to find the class {}", name);
    }

    pub fn find_identifier(&self, name: &str) -> Option<&parse::tree::Param<'def>> {
        for i in (0..self.levels.len()).rev() {
            let level = self.levels.get(i).unwrap();
            match level.enclosing_opt {
                Some(LevelEnclosing::Class(class)) => {
                    let class = unsafe { &*class };
                    for param in &unsafe { &*class.parse }.params {
                        if param.name.fragment == name {
                            return Some(param);
                        }
                    }
                },
                Some(LevelEnclosing::Method(method)) => {
                    let method = unsafe { &*method };
                    for param in &unsafe { &*method.parse }.params {
                        if param.name.fragment == name {
                            return Some(param);
                        }
                    }
                },
                _ => (),
            }
        }

        panic!("Unable to find the class {}", name);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Level<'def> {
    pub enclosing_opt: Option<LevelEnclosing<'def>>
}

#[derive(Debug, PartialEq, Clone)]
pub enum LevelEnclosing<'def> {
    Root(*const Root<'def>),
    Class(*const index::tree::Class<'def>),
    Method(*const index::tree::Method<'def>),
}
