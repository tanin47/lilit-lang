use parse::{expr, ParseResult, Tokens};
use parse::combinator::{separated_list, symbol};
use parse::tree::{Expr, Invoke};
use tokenize::span::Span;
use tokenize::token::Token;
use std::cell::Cell;
use parse::expr::atom::identifier;

pub fn parse<'def, 'r>(
    original: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Invoke<'def>> {
    let (input, name) = identifier::parse_span(original)?;

    if let Ok((input, invoke)) = parse_tail(name, input) {
        Ok((input, invoke))
    } else {
        Err(original)
    }
}

pub fn parse_tail<'def, 'r>(
    name: Span<'def>,
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Invoke<'def>> {
    if let Ok((input, args)) = parse_args(input) {
        Ok((
            input,
            Invoke {
                invoker_opt: None,
                name,
                args,
                method_def: None,
            }
        ))
    } else {
        Err(input)
    }
}

pub fn parse_args<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Vec<Expr<'def>>> {
    let (input, _) = symbol('(')(input)?;
    let (input, args) = separated_list(symbol(','), expr::parse)(input)?;
    let (input, _) = symbol(')')(input)?;
    Ok((input, args))
}

#[cfg(test)]
mod tests {
    use parse::expr::atom::invoke;
    use parse::Tokens;
    use parse::tree::{Expr, Invoke, LiteralString};
    use test_common::{generate_tokens, span};
    use std::cell::{Cell, RefCell};

    #[test]
    fn test_simple() {
        assert_eq!(
            invoke::parse(&generate_tokens(
                r#"
func("a", "b")
           "#
            )),
            Ok((
                &[] as Tokens,
                Invoke {
                    invoker_opt: None,
                    name: span(1, 1, "func"),
                    args: vec![
                        Expr::String(Box::new(LiteralString { span: span(1, 6, "\"a\""), instance: None })),
                        Expr::String(Box::new(LiteralString { span: span(1, 11, "\"b\""), instance: None })),
                    ],
                    method_def: None,
                }
            ))
        );
    }
}
