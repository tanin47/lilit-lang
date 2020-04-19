use parse::{ParseResult, Tokens};
use parse::tree::NewInstance;
use tokenize::span::Span;
use tokenize::token::Token;
use parse::expr::atom::invoke;
use std::cell::Cell;

pub fn parse<'def, 'r>(
    original: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, NewInstance<'def>> {
    let (input, name) = parse_capitalize(original)?;

    if let Ok((input, args)) = invoke::parse_args(input) {
        Ok((
            input,
            NewInstance {
                name_opt: Some(name),
                args,
                class_def: Cell::new(None),
            }
        ))
    } else {
        Err(original)
    }
}

fn parse_capitalize<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Span<'def>> {
    if let Token::Capitalize(name) = input[0] {
        Ok((&input[1..], name))
    } else {
        Err(input)
    }
}

#[cfg(test)]
mod tests {
    use parse::expr::atom::new_instance;
    use parse::Tokens;
    use parse::tree::{Expr, LiteralString, NewInstance, Int};
    use test_common::{generate_tokens, span};
    use std::cell::{Cell, RefCell};

    #[test]
    fn test_simple() {
        assert_eq!(
            new_instance::parse(&generate_tokens(
                r#"
Int("a", 5)
           "#
            )),
            Ok((
                &[] as Tokens,
                NewInstance {
                    name_opt: Some(span(1, 1, "Int")),
                    args: vec![
                        Expr::String(Box::new(LiteralString { span: span(1, 5, "\"a\""), instance: RefCell::new(None) })),
                        Expr::Int(Box::new(Int { span: span(1, 10, "5"), instance: RefCell::new(None) })),
                    ],
                    class_def: Cell::new(None),
                }
            ))
        );
    }
}
