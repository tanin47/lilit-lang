use parse::{Tokens, ParseResult, expr};
use parse::tree::{Expr, Assignment};
use parse::expr::atom::identifier;
use parse::combinator::symbol;
use std::cell::Cell;
use parse::expr::level_010;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Expr<'def>> {
    if let Ok((input, e)) = parse_assignment(input) {
        Ok((input, e))
    } else {
        level_010::parse(input)
    }
}

fn parse_assignment<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, name) = identifier::parse_span(input)?;
    let (input, _) = symbol('=')(input)?;
    let (input, expr) = expr::parse(input)?;

    Ok((
        input,
        Expr::Assignment(Box::from(Assignment {
            name,
            expr: Box::new(expr),
            tpe: Cell::new(None),
            llvm: Cell::new(None),
        }))
    ))
}
