use parse::{Tokens, ParseResult};
use parse::tree::Identifier;
use tokenize::token::Token;
use std::cell::RefCell;
use tokenize::span::Span;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Identifier<'def>> {
    let (input, name) = parse_span(input)?;
    Ok((input, Identifier { name: Some(name), def_opt: RefCell::new(None) }))
}

pub fn parse_span<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Span<'def>> {
    if let Token::Identifier(name) = input[0] {
        Ok((&input[1..], name))
    } else {
        Err(input)
    }
}
