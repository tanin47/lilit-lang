use parse::{Tokens, ParseResult};
use tokenize::span::Span;
use parse::tree::Expr;

pub mod atom;
pub mod level_010;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, expr) = level_010::parse(input)?;
    Ok((input, expr))
}