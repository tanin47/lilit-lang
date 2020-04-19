use parse::tree::Int;
use parse::{Tokens, ParseResult};
use tokenize::token::Token;
use std::cell::RefCell;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Int<'def>> {
    if let Token::Int(span) = &input[0] {
        Ok((&input[1..], Int { span: *span, instance: None }))
    } else {
        Err(input)
    }
}
