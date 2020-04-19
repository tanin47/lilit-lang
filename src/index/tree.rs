use parse;

#[derive(Debug, PartialEq, Clone)]
pub struct Root<'a> {
    pub items: Vec<RootItem<'a>>,
}

impl <'a> Root<'a> {
    pub fn find_method(&self, name: &str) -> &parse::tree::Method<'a> {
        for item in &self.items {
           if let RootItem::Method(method)  = item {
               let method = unsafe { &*method.parse };
               if method.name.fragment == name {
                   return method;
               }
           }
        }

        panic!("Unable to find the method {}", name)
    }

    pub fn find_class(&self, name: &str) -> &parse::tree::Class<'a> {
        for item in &self.items {
            if let RootItem::Class(class) = item {
                let class = unsafe { &*class.parse };
                if class.name.fragment == name {
                    return class;
                }
            }
        }

        panic!("Unable to find the class {}", name);
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
