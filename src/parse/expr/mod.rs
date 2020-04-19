use parse::{Tokens, ParseResult};
use tokenize::span::Span;
use parse::tree::Expr;

pub mod atom;
pub mod level_010;
pub mod level_016;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, expr) = level_016::parse(input)?;
    Ok((input, expr))
}