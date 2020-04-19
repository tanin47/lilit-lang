use parse::tree::{CompilationUnit, Class, Method, Assignment, Param, IdentifierSource};
use index::tree::{Root, RootItem};
use ::{index, parse};

#[derive(Debug, PartialEq, Clone)]
pub struct Scope<'def> {
    pub levels: Vec<Level<'def>>,
}

impl <'def> Scope<'def> {
    pub fn enter(&mut self) {
        self.levels.push(Level { enclosing_opt: None, assignments: vec![] });
    }

    pub fn enter_root(&mut self, root: &Root<'def>) {
        self.levels.push(Level { enclosing_opt: Some(LevelEnclosing::Root(root)), assignments: vec![] });
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
                                self.levels.push(Level { enclosing_opt: Some(LevelEnclosing::Class(candidate)), assignments: vec![] });
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
                                self.levels.push(Level { enclosing_opt: Some(LevelEnclosing::Method(candidate)), assignments: vec![] });
                                return;
                            }
                        }
                    }
                },
                Some(LevelEnclosing::Class(class)) => {
                    let class = unsafe { &*class };
                    for candidate in &class.methods {
                       if candidate.parse == method  {
                           self.levels.push(Level { enclosing_opt: Some(LevelEnclosing::Method(candidate)), assignments: vec![] });
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

    pub fn find_identifier(&self, name: &str) -> Option<IdentifierSource<'def>> {
        for i in (0..self.levels.len()).rev() {
            let level = self.levels.get(i).unwrap();

            for assignment in &level.assignments {
               let assignment = unsafe { &**assignment };
                if assignment.name.fragment == name {
                    return Some(IdentifierSource::Assignment(assignment));
                }
            }

            match level.enclosing_opt {
                Some(LevelEnclosing::Class(class)) => {
                    let class = unsafe { &*class };
                    for param in &unsafe { &*class.parse }.params {
                        if param.name.unwrap().fragment == name {
                            return Some(IdentifierSource::Param(param));
                        }
                    }
                },
                Some(LevelEnclosing::Method(method)) => {
                    let method = unsafe { &*method };
                    for param in &unsafe { &*method.parse }.params {
                        if param.name.map(|x|x.fragment) == Some(name) {
                            return Some(IdentifierSource::Param(param));
                        }
                    }
                },
                _ => (),
            }
        }

        panic!("Unable to find the class {}", name);
    }

    pub fn find_parent_method(&self) -> &Method<'def> {
        for i in (0..self.levels.len()).rev() {
            let level = self.levels.get(i).unwrap();

            match level.enclosing_opt {
                Some(LevelEnclosing::Method(method)) => {
                    return unsafe { &*(&*method).parse };
                },
                _ => (),
            }
        }

        panic!("Unable to find the parent method");

    }

    pub fn add_var(&mut self, assignment: &Assignment<'def>) {
        self.levels.last_mut().unwrap().assignments.push(assignment);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Level<'def> {
    pub enclosing_opt: Option<LevelEnclosing<'def>>,
    pub assignments: Vec<* const Assignment<'def>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LevelEnclosing<'def> {
    Root(*const Root<'def>),
    Class(*const index::tree::Class<'def>),
    Method(*const index::tree::Method<'def>),
}
