use parse;

#[derive(Debug, PartialEq, Clone)]
pub struct Root<'a> {
    pub items: Vec<RootItem<'a>>,
}

impl <'a> Root<'a> {
    pub fn find_method(&self, name: &str) -> Option<&Method<'a>> {
        for item in &self.items {
           if let RootItem::Method(method)  = item {
               if unsafe { &*method.parse }.name.fragment == name {
                   return Some(method);
               }
           }
        }

        None
    }

    pub fn find_class(&self, name: &str) -> Option<&Class<'a>> {
        for item in &self.items {
            if let RootItem::Class(class) = item {
                if unsafe { &*class.parse }.name.fragment == name {
                    return Some(class)
                }
            }
        }

        None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RootItem<'a> {
    Class(Class<'a>),
    Method(Method<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Class<'a> {
    pub methods: Vec<Method<'a>>,
    pub parse: *const parse::tree::Class<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ClassItem<'a> {
    Class(Class<'a>),
    Method(Method<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Method<'a> {
    pub parse: *const parse::tree::Method<'a>,
}
