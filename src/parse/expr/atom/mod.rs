use parse::{Tokens, ParseResult};
use parse::tree::Expr;

pub mod identifier;
pub mod int;
pub mod invoke;
pub mod literal_string;
pub mod new_instance;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Expr<'def>> {
    if let Ok((input, i)) = literal_string::parse(input) {
        Ok((input, Expr::String(Box::new(i))))
    } else if let Ok((input, i)) = invoke::parse(input) {
        Ok((input, Expr::Invoke(Box::new(i))))
    } else if let Ok((input, i)) = new_instance::parse(input) {
        Ok((input, Expr::NewInstance(Box::new(i))))
    } else if let Ok((input, i)) = int::parse(input) {
        Ok((input, Expr::Int(Box::new(i))))
    } else if let Ok((input, i)) = identifier::parse(input) {
        Ok((input, Expr::Identifier(Box::new(i))))
    } else {
        Err(input)
    }
}