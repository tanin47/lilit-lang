use tokenize::span::Span;
use std::cell::{Cell, RefCell};
use inkwell::types::{StructType, BasicTypeEnum};
use inkwell::values::{FunctionValue, PointerValue};
use inkwell::AddressSpace;
use inkwell::context::Context;

#[derive(Debug, PartialEq, Clone)]
pub struct CompilationUnit<'a> {
    pub items: Vec<CompilationUnitItem<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CompilationUnitItem<'a> {
    Class(Class<'a>),
    Method(Method<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Class<'a> {
    pub name: Span<'a>,
    pub generics: Vec<GenericDef<'a>>,
    pub params: Vec<Param<'a>>,
    pub methods: Vec<Method<'a>>,
    pub llvm: Cell<Option<StructType>>,
    pub llvm_native: Cell<Option<StructType>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct GenericDef<'a> {
    pub name: Span<'a>,
    pub index: usize,
}

impl <'a> GenericDef<'a> {
    pub fn init(name: Span, index: usize) -> GenericDef {
        GenericDef {
            name,
            index,
        }
    }
}

impl <'a> Class<'a> {
    pub fn find_method(&self, name: &str) -> &Method<'a> {
        for method in &self.methods {
           if method.name.fragment == name {
               return method;
           }
        }

        panic!("Unable to find the method {} in the class {}", name, self.name.fragment)
    }

    pub fn find_param(&self, name: &str) -> &Param<'a> {
        for param in &self.params {
            if param.name.unwrap().fragment == name {
                return param;
            }
        }

        panic!("Unable to find the param {} in the class {}", name, self.name.fragment)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Method<'a> {
    pub name: Span<'a>,
    pub params: Vec<Param<'a>>,
    pub exprs: Vec<Expr<'a>>,
    pub return_type: Type<'a>,
    pub parent_class: Option<*const Class<'a>>,
    pub llvm: Cell<Option<FunctionValue>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param<'a> {
    pub name: Option<Span<'a>>,
    pub tpe: Type<'a>,
    pub is_varargs: bool,
    pub index: usize,
    pub parent: Option<ParamParent<'a>>,
    pub llvm: Cell<Option<PointerValue>>,
}

impl <'a> Param<'a> {
    pub fn init<'b>(name: Option<Span<'b>>, tpe: Type<'b>, is_varargs: bool, index: usize) -> Param<'b> {
        Param {
            name,
            tpe,
            is_varargs,
            index,
            parent: None,
            llvm: Cell::new(None),
        }
    }

    // TODO: how should we handle this?
    pub fn get_llvm_type(&self, context: &Context) -> BasicTypeEnum {
        match self.tpe.kind.as_ref() {
            TypeKind::Class(c) => {
                let param_class = unsafe { &*c.class_def.unwrap() };
                BasicTypeEnum::PointerType(if self.is_varargs {
                    param_class.llvm.get().unwrap().ptr_type(AddressSpace::Generic).ptr_type(AddressSpace::Generic)
                } else {
                    param_class.llvm.get().unwrap().ptr_type(AddressSpace::Generic)
                })
            }
            TypeKind::Generic(g) => {
                BasicTypeEnum::PointerType(if self.is_varargs {
                    context.void_type().ptr_type(AddressSpace::Generic).ptr_type(AddressSpace::Generic)
                } else {
                    context.void_type().ptr_type(AddressSpace::Generic)
                })
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParamParent<'a> {
    Class(*const Class<'a>),
    Method(*const Method<'a>)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Type<'a> {
    pub span: Option<Span<'a>>,
    pub kind: Box<TypeKind<'a>>,
}

impl <'a> Type<'a> {
    pub fn init(span: Option<Span>) -> Type {
        Type { span, kind: Box::new(TypeKind::Class(ClassType { class_def: None, generics: vec![] })) }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeKind<'a> {
    Class(ClassType<'a>),
    Generic(GenericType<'a>)
}

impl <'a> TypeKind<'a> {
    pub fn init_class_type(class_def: *const Class<'a>) -> TypeKind {
        TypeKind::Class(ClassType {
            class_def: Some(class_def),
            generics: vec![]
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassType<'a> {
    pub class_def: Option<* const Class<'a>>, // TODO: remove Some(..)
    pub generics: Vec<Type<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GenericType<'a> {
    pub generic_def: Option<* const GenericDef<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a> {
    Assignment(Box<Assignment<'a>>),
    Char(Box<Char<'a>>),
    Identifier(Box<Identifier<'a>>),
    Int(Box<Int<'a>>),
    Invoke(Box<Invoke<'a>>),
    MemberAccess(Box<MemberAccess<'a>>),
    NativeChar(Box<NativeChar>),
    NativeInt(Box<NativeInt>),
    NativeString(Box<NativeString>),
    NewInstance(Box<NewInstance<'a>>),
    String(Box<LiteralString<'a>>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assignment<'a> {
    pub name: Span<'a>,
    pub expr: Box<Expr<'a>>,
    pub tpe: Option<TypeKind<'a>>,
    pub llvm: Cell<Option<PointerValue>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Identifier<'a> {
    pub name: Option<Span<'a>>,
    pub source: Option<IdentifierSource<'a>>
}

#[derive(Debug, PartialEq, Clone)]
pub enum IdentifierSource<'a> {
    Assignment(*const Assignment<'a>),
    Param(*const Param<'a>),
    ClassParam(Box<MemberAccess<'a>>),
}

impl <'a> IdentifierSource<'a> {
    pub fn get_type(&self) -> *const TypeKind<'a> {
        match self {
            IdentifierSource::Assignment(a) => unsafe { (&**a) }.tpe.as_ref().unwrap(),
            IdentifierSource::Param(p) => unsafe { (&**p) }.tpe.kind.as_ref(),
            IdentifierSource::ClassParam(p) => unsafe { (&*p.param_def.unwrap()).tpe.kind.as_ref() },
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Invoke<'a> {
    pub invoker_opt: Option<Expr<'a>>,
    pub name: Span<'a>,
    pub args: Vec<Expr<'a>>,
    pub method_def: Option<* const Method<'a>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct MemberAccess<'a> {
    pub parent: Expr<'a>,
    pub name: Option<Span<'a>>,
    pub param_def: Option<* const Param<'a>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct NewInstance<'a> {
    pub name_opt: Option<Span<'a>>,
    pub generics: Vec<Type<'a>>,
    pub args: Vec<Expr<'a>>,
    pub tpe: Option<TypeKind<'a>>
}

impl<'a> NewInstance<'a> {
    pub fn init<'b>(name_opt: Option<Span<'b>>, generics: Vec<Type<'b>>, args: Vec<Expr<'b>>) -> NewInstance<'b> {
        NewInstance {
            name_opt,
            generics,
            args,
            tpe: None
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LiteralString<'a> {
    pub span: Span<'a>,
    pub instance: Option<Box<NewInstance<'a>>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Int<'a> {
    pub span: Span<'a>,
    pub instance: Option<Box<NewInstance<'a>>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Char<'a> {
    pub span: Span<'a>,
    pub instance: Option<Box<NewInstance<'a>>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct NativeChar {
    pub value: char
}

#[derive(Debug, PartialEq, Clone)]
pub struct NativeInt {
    pub value: i64
}

#[derive(Debug, PartialEq, Clone)]
pub struct NativeString {
    pub value: String
}
