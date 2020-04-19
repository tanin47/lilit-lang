use parse::{Tokens, ParseResult};
use parse::tree::{LiteralString, Char};
use tokenize::token::Token;
use std::cell::RefCell;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Char<'def>> {
    if let Token::Char(span) = &input[0] {
        Ok((&input[1..], Char { span: *span, instance: RefCell::new(None) }))
    } else {
        Err(input)
    }
}
