use tokenize::span::Span;
use std::cell::{Cell, RefCell};
use inkwell::types::{StructType, BasicTypeEnum};
use inkwell::values::{FunctionValue, PointerValue};
use inkwell::AddressSpace;

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
    pub params: Vec<Param<'a>>,
    pub methods: Vec<Method<'a>>,
    pub llvm: Cell<Option<StructType>>,
    pub llvm_native: Cell<Option<StructType>>
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
    pub parent_class: Cell<Option<*const Class<'a>>>,
    pub llvm: Cell<Option<FunctionValue>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param<'a> {
    pub name: Option<Span<'a>>,
    pub tpe: Type<'a>,
    pub is_varargs: bool,
    pub index: usize,
    pub parent: Cell<Option<ParamParent<'a>>>,
    pub llvm: Cell<Option<PointerValue>>,
}

impl <'a> Param<'a> {
    pub fn get_llvm_type(&self) -> BasicTypeEnum {
        let param_class = unsafe { &*self.tpe.def_opt.get().unwrap() };
        BasicTypeEnum::PointerType(if self.is_varargs {
            param_class.llvm.get().unwrap().ptr_type(AddressSpace::Generic).ptr_type(AddressSpace::Generic)
        } else {
            param_class.llvm.get().unwrap().ptr_type(AddressSpace::Generic)
        })
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
    pub def_opt: Cell<Option<* const Class<'a>>>
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
    pub tpe: Cell<Option<*const Class<'a>>>,
    pub llvm: Cell<Option<PointerValue>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Identifier<'a> {
    pub name: Option<Span<'a>>,
    pub def_opt: RefCell<Option<IdentifierSource<'a>>>
}

#[derive(Debug, PartialEq, Clone)]
pub enum IdentifierSource<'a> {
    Assignment(*const Assignment<'a>),
    Param(*const Param<'a>),
    ClassParam(Box<MemberAccess<'a>>),
}

impl <'a> IdentifierSource<'a> {
    pub fn get_type(&self) -> *const Class<'a> {
        match self {
            IdentifierSource::Assignment(a) => unsafe { &*(&**a) .tpe.get().unwrap() },
            IdentifierSource::Param(p) => unsafe { &*(&**p).tpe.def_opt.get().unwrap() }
            IdentifierSource::ClassParam(p) => unsafe { &*(&*p.def_opt.get().unwrap()).tpe.def_opt.get().unwrap() }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Invoke<'a> {
    pub invoker_opt: Option<Expr<'a>>,
    pub name: Span<'a>,
    pub args: Vec<Expr<'a>>,
    pub def_opt: Cell<Option<* const Method<'a>>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct MemberAccess<'a> {
    pub parent: Expr<'a>,
    pub name: Option<Span<'a>>,
    pub def_opt: Cell<Option<* const Param<'a>>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct NewInstance<'a> {
    pub name_opt: Option<Span<'a>>,
    pub args: Vec<Expr<'a>>,
    // TODO(tanin): this should refer to a constructor
    pub def_opt: Cell<Option<* const Class<'a>>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct LiteralString<'a> {
    pub span: Span<'a>,
    pub instance: RefCell<Option<NewInstance<'a>>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Int<'a> {
    pub span: Span<'a>,
    pub instance: RefCell<Option<NewInstance<'a>>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Char<'a> {
    pub span: Span<'a>,
    pub instance: RefCell<Option<NewInstance<'a>>>
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
