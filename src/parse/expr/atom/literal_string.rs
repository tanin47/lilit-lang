use parse::{Tokens, ParseResult};
use parse::tree::LiteralString;
use tokenize::token::Token;
use std::cell::RefCell;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, LiteralString<'def>> {
    if let Token::String(span) = &input[0] {
        Ok((&input[1..], LiteralString { span: *span, instance: RefCell::new(None) }))
    } else {
        Err(input)
    }
}
