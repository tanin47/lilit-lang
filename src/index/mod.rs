use ::{LilitFile, parse};
use index::tree::{Class, Method, Root, RootItem};
use parse::tree::CompilationUnitItem;

pub mod tree;

pub fn build<'def, 'r, 'l>(
    files: &'r [&'l LilitFile<'def>],
) -> Root<'def> {
    let mut items = vec![];

    for file in files {
        items.append(&mut build_file(*file));
    }

    Root { items }
}

fn build_file<'def, 'r>(
    file: &'r LilitFile<'def>
) -> Vec<RootItem<'def>> {
    let mut items = vec![];
    for item in &file.unit.items {
        items.push(
            match item {
                CompilationUnitItem::Class(class) => RootItem::Class(build_class(class)),
                CompilationUnitItem::Method(method) => RootItem::Method(build_method(method)),
            }
        );
    }

    items
}

fn build_class<'def, 'r>(
    class: &'r parse::tree::Class<'def>
) -> Class<'def> {
    let mut methods = vec![];

    for m in &class.methods {
        methods.push(build_method(m));
    }

    Class {
        methods,
        parse: class as *const parse::tree::Class<'def>,
    }
}

fn build_method<'def, 'r>(
    method: &'r parse::tree::Method<'def>
) -> Method<'def> {
    Method {
        parse: method as *const parse::tree::Method<'def>,
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use index::build;
    use index::tree::{Class, Method, Root, RootItem};
    use parse;
    use parse::tree::{CompilationUnit, Type};
    use test_common::span2;
    use std::cell::Cell;

    #[test]
    fn test_simple() {
        let contents = vec![
            r#"
def main(): Number
end
            "#,
            r#"
class Test
  def test(): Number
  end
end
            "#,
        ];
        let files = contents.iter().map(|&content| unwrap!(Ok, parse::apply(content.trim(), ""))).collect::<Vec<_>>();
        let root = build(files.iter().map(|file| file.deref()).collect::<Vec<_>>().deref());

        assert_eq!(
            root.find_method("main"),
            &parse::tree::Method {
                name: span2(1, 5, "main", files.get(0).unwrap().deref()),
                params: vec![],
                exprs: vec![],
                return_type: Type { span: Some(span2(1, 13, "Number", files.get(0).unwrap().deref())), class_def: Cell::new(None) },
                parent_class: Cell::new(None),
                llvm: Cell::new(None)
            }
        );
        assert_eq!(
            root.find_class("Test"),
            &parse::tree::Class {
                name: span2(1, 7, "Test", files.get(1).unwrap().deref()),
                params: vec![],
                methods: vec![
                    parse::tree::Method {
                        name: span2(2, 7, "test", files.get(1).unwrap().deref()),
                        params: vec![],
                        exprs: vec![],
                        return_type: Type { span: Some(span2(2, 15, "Number", files.get(1).unwrap().deref())), class_def: Cell::new(None) },
                        parent_class: Cell::new(None),
                        llvm: Cell::new(None),
                    }
                ],
                llvm: Cell::new(None),
                llvm_native: Cell::new(None),
            }
        );
        assert_eq!(
            root,
            Root {
                items: vec![
                    RootItem::Method(Method {
                        parse: root.find_method("main"),
                    }),
                    RootItem::Class(Class {
                        methods: vec![
                            Method {
                                parse: root.find_class("Test").find_method("test"),
                            }
                        ],
                        parse: root.find_class("Test"),
                    }),
                ]
            }
        )
    }
}
