use parse::tree::{Method, ParamParent, Class, Param, Type, TypeKind, ClassType, GenericType};
use analyse::{expr, tpe};
use analyse::scope::Scope;
use analyse::def::params;
use std::cell::Cell;

pub fn apply<'def>(
    method: &mut Method<'def>,
    parent_class: Option<*const Class<'def>>,
    scope: &mut Scope<'def>
) {
    scope.enter_method(method);

    if let Some(parent_class) = parent_class {
        let mut generics = vec![];
        let parent_class = unsafe { &*parent_class };

        for g in &parent_class.generics {
            generics.push(Type {
                span: None,
                kind: Box::new(TypeKind::Generic(GenericType {
                    generic_def: Some(g)
                }))
            });
        }

        method.params.insert(0, Param {
            name: None,
            tpe: Type {
                span: None,
                kind: Box::new(TypeKind::Class(ClassType {
                    class_def: Some(parent_class),
                    generics,
                }))
            },
            is_varargs: false,
            index: 0,
            parent: Some(ParamParent::Method(method)),
            llvm: Cell::new(None)
        })
    }

    let parent = ParamParent::Method(method);
    params::apply(&mut method.params, parent, scope);
    tpe::apply(&mut method.return_type, scope);

    for e in &mut method.exprs {
        expr::apply(e, scope);
    }
    scope.leave();
}

#[cfg(test)]
mod tests {
    use std::ops::{Deref, DerefMut};

    use index::build;
    use parse;
    use test_common::span2;
    use analyse::apply;
    use std::cell::Cell;
    use parse::tree::{CompilationUnit, CompilationUnitItem, Class, Method, Type, TypeKind, GenericType, GenericDef, Param, ClassType, ParamParent, Identifier, Expr, IdentifierSource, MemberAccess, Invoke, NewInstance, Int, NativeInt};

    #[test]
    fn test_generics() {
        let content = r#"
class Number
end

class Test[T](member: T)
  def get(): T
    member
  end
end

def main(): Number
  Test[Number](Number()).get()
end
        "#;
        let mut file = unwrap!(Ok, parse::apply(content.trim(), ""));
        let root = build(&[file.deref()]);

        apply(&mut [file.deref_mut()], &root);

        let class = root.find_class("Test");
        let method = class.find_method("get");
        assert_eq!(
            file.unit,
            CompilationUnit {
                items: vec![
                    CompilationUnitItem::Class(Class {
                        name: span2(1, 7, "Number", file.deref()),
                        generics: vec![],
                        params: vec![],
                        methods: vec![],
                        llvm: Cell::new(None),
                        llvm_native: Cell::new(None)
                    }),
                    CompilationUnitItem::Class(Class {
                        name: span2(4, 7, "Test", file.deref()),
                        generics: vec![
                            GenericDef {
                                name: span2(4, 12, "T", file.deref()),
                                index: 0
                            }
                        ],
                        params: vec![
                            Param {
                                name: Some(span2(4, 15, "member", file.deref())),
                                tpe: Type {
                                    span: Some(span2(4, 23, "T", file.deref())),
                                    kind: Box::new(TypeKind::Generic(GenericType {
                                        generic_def: Some(class.generics.get(0).unwrap())
                                    }))
                                },
                                is_varargs: false,
                                index: 0,
                                parent: Some(ParamParent::Class(class)),
                                llvm: Cell::new(None)
                            }
                        ],
                        methods: vec![
                            Method {
                                name: span2(5, 7, "get", file.deref()),
                                params: vec![
                                    Param {
                                        name: None,
                                        tpe: Type {
                                            span: None,
                                            kind: Box::new(TypeKind::Class(ClassType {
                                                class_def: Some(class),
                                                generics: vec![
                                                    Type {
                                                        span: None,
                                                        kind: Box::new(TypeKind::Generic(GenericType {
                                                            generic_def: Some(class.generics.get(0).unwrap())
                                                        }))
                                                    }
                                                ]
                                            }))
                                        },
                                        is_varargs: false,
                                        index: 0,
                                        parent: Some(ParamParent::Method(method)),
                                        llvm: Cell::new(None)
                                    }
                                ],
                                exprs: vec![
                                    Expr::Identifier(Box::from(Identifier {
                                        name: Some(span2(6, 5, "member", file.deref())),
                                        source: Some(IdentifierSource::ClassParam(Box::new(MemberAccess {
                                            parent: Expr::Identifier(Box::from(Identifier {
                                                name: None,
                                                source: Some(IdentifierSource::Param(method.params.get(0).unwrap()))
                                            })),
                                            name: None,
                                            param_def: Some(class.params.get(0).unwrap())
                                        })))
                                    }))
                                ],
                                return_type: Type {
                                    span: Some(span2(5, 14, "T", file.deref())),
                                    kind: Box::new(TypeKind::Generic(GenericType {
                                        generic_def: Some(class.generics.get(0).unwrap())
                                    }))
                                },
                                parent_class: None,
                                llvm: Cell::new(None)
                            }
                        ],
                        llvm: Cell::new(None),
                        llvm_native: Cell::new(None)
                    }),
                    CompilationUnitItem::Method(Method {
                        name: span2(10, 5, "main", file.deref()),
                        params: vec![],
                        exprs: vec![
                            Expr::Invoke(Box::from(Invoke {
                                invoker_opt: Some(Expr::NewInstance(Box::from(NewInstance {
                                    name_opt: Some(span2(11, 3, "Test", file.deref())),
                                    generics: vec![
                                        Type {
                                            span: Some(span2(11, 8, "Number", file.deref())),
                                            kind: Box::new(TypeKind::Class(ClassType {
                                                class_def: Some(root.find_class("Number")),
                                                generics: vec![]
                                            }))
                                        }
                                    ],
                                    args: vec![
                                        Expr::NewInstance(Box::from(NewInstance {
                                            name_opt: Some(span2(11, 16, "Number", file.deref())),
                                            generics: vec![],
                                            args: vec![],
                                            tpe: Some(TypeKind::Class(ClassType {
                                                class_def: Some(root.find_class("Number")),
                                                generics: vec![]
                                            }))
                                        }))
                                    ],
                                    tpe: Some(TypeKind::Class(ClassType {
                                        class_def: Some(class),
                                        generics: vec![
                                            Type {
                                                span: Some(span2(11, 8, "Number", file.deref())),
                                                kind: Box::new(TypeKind::Class(ClassType {
                                                    class_def: Some(root.find_class("Number")),
                                                    generics: vec![]
                                                }))
                                            }
                                        ]
                                    }))
                                }))),
                                name: span2(11, 26, "get", file.deref()),
                                args: vec![],
                                method_def: Some(method),
                            }))
                        ],
                        return_type: Type {
                            span: Some(span2(10, 13, "Number", file.deref())),
                            kind: Box::new(TypeKind::Class(ClassType {
                                class_def: Some(root.find_class("Number")),
                                generics: vec![]
                            }))
                        },
                        parent_class: None,
                        llvm: Cell::new(None)
                    })
                ]
            }
        )
    }
}
