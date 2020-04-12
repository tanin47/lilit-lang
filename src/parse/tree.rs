use tokenize::span::Span;
use std::cell::{Cell, RefCell};
use inkwell::types::StructType;
use inkwell::values::{FunctionValue, PointerValue};

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
    pub llvm: Cell<Option<StructType>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Method<'a> {
    pub name: Span<'a>,
    pub params: Vec<Param<'a>>,
    pub exprs: Vec<Expr<'a>>,
    pub return_type: Type<'a>,
    pub llvm: Cell<Option<FunctionValue>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param<'a> {
    pub name: Span<'a>,
    pub tpe: Type<'a>,
    pub is_varargs: bool,
    pub index: usize,
    pub parent: Cell<Option<ParamParent<'a>>>,
    pub llvm: Cell<Option<PointerValue>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParamParent<'a> {
    Class(*const Class<'a>),
    Method(*const Method<'a>)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Type<'a> {
    pub span: Span<'a>,
    pub def_opt: Cell<Option<* const Class<'a>>>
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a> {
    String(Box<LiteralString<'a>>),
    NativeString(Box<NativeString>),
    NativeInt(Box<NativeInt>),
    Int(Box<Int<'a>>),
    Invoke(Box<Invoke<'a>>),
    MemberAccess(Box<MemberAccess<'a>>),
    NewInstance(Box<NewInstance<'a>>),
    Identifier(Box<Identifier<'a>>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Identifier<'a> {
    pub name: Span<'a>,
    pub def_opt: Cell<Option<* const Param<'a>>>
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
    pub name: Span<'a>,
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
pub struct NativeInt {
    pub value: i64
}

#[derive(Debug, PartialEq, Clone)]
pub struct NativeString {
    pub value: String
}
