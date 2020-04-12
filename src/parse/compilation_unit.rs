use parse::combinator::{many1, opt};
use parse::def::{method, class};
use parse::tree::{CompilationUnit, CompilationUnitItem};
use parse::{ParseResult, Tokens};
use tokenize::token::Token;

pub fn parse_item<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, CompilationUnitItem<'def>> {

    if let Ok((input, _)) = method::parse_prefix(input) {
        let (input, method) = method::parse_tail(input)?;
        Ok((input, CompilationUnitItem::Method(method)))
    } else if let Ok((input, _)) = class::parse_prefix(input) {
        let (input, class) = class::parse_tail(input)?;
        Ok((input, CompilationUnitItem::Class(class)))
    } else {
        Err(input)
    }
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, CompilationUnit<'def>> {
    let (input, items) = many1(|i| parse_item(i))(input)?;

    Ok((
        input,
        CompilationUnit {
            items,
        },
    ))
}