use emit::{Emitter, Value};
use parse::tree::Expr;
use emit::expr::int::IntEmitter;
use emit::expr::native_int::NativeIntEmitter;
use emit::expr::new_instance::NewInstanceEmitter;
use emit::expr::member_access::MemberAccessEmitter;
use emit::expr::identifier::IdentifierEmitter;
use emit::expr::literal_string::LiteralStringEmitter;
use emit::expr::native_string::NativeStringEmitter;
use emit::expr::invoke::InvokeEmitter;
use emit::expr::literal_char::LiteralCharEmitter;
use emit::expr::native_char::NativeCharEmitter;
use emit::expr::assignment::AssignmentEmitter;

pub mod assignment;
pub mod identifier;
pub mod int;
pub mod invoke;
pub mod literal_string;
pub mod literal_char;
pub mod member_access;
pub mod native_char;
pub mod native_int;
pub mod native_string;
pub mod new_instance;

pub trait ExprEmitter {
    fn apply_expr<'def>(&self, expr: &Expr<'def>) -> Value<'def>;
}

impl ExprEmitter for Emitter<'_> {
    fn apply_expr<'def>(&self, expr: &Expr<'def>) -> Value<'def> {
        match expr {
            Expr::Assignment(i) => self.apply_assignment(i),
            Expr::Char(i) => self.apply_literal_char(i),
            Expr::Identifier(i) => self.apply_identifier(i),
            Expr::Int(i) => self.apply_int(i),
            Expr::Invoke(i) => self.apply_invoke(i),
            Expr::MemberAccess(i) => self.apply_member_access(i),
            Expr::NativeChar(i) => self.apply_native_char(i),
            Expr::NativeInt(i) => self.apply_native_int(i),
            Expr::NativeString(i) => self.apply_native_string(i),
            Expr::NewInstance(i) => self.apply_new_instance(i),
            Expr::String(i) => self.apply_literal_string(i),
        }
    }
}