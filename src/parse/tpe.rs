use parse::{Tokens, ParseResult};
use parse::tree::Type;
use parse::combinator::capitalize;
use std::cell::Cell;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Type<'def>> {
    let (input, name) = capitalize(input)?;

    Ok((input, Type { span: name, def_opt: Cell::new(None) }))
}
