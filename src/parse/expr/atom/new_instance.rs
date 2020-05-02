use parse::{ParseResult, Tokens, tpe};
use parse::tree::{NewInstance, GenericType, Type};
use tokenize::span::Span;
use tokenize::token::Token;
use parse::expr::atom::invoke;
use std::cell::Cell;
use parse::combinator::{symbol, separated_list, opt};

pub fn parse<'def, 'r>(
    original: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, NewInstance<'def>> {
    let (input, name) = parse_capitalize(original)?;
    let (input, generic_args_opt) = opt(parse_generic_args)(input)?;

    if let Ok((input, args)) = invoke::parse_args(input) {
        Ok((
            input,
            NewInstance::init(Some(name), generic_args_opt.unwrap_or(vec![]), args)
        ))
    } else {
        Err(original)
    }
}

fn parse_generic_args<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Vec<Type<'def>>> {
    let (input, _) = symbol('[')(input)?;
    let (input, types) = separated_list(symbol(','), tpe::parse)(input)?;
    let (input, _) = symbol(']')(input)?;

    Ok((input, types))
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
    use parse::tree::{Expr, LiteralString, NewInstance, Int, Type};
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
                NewInstance::init(
                    Some(span(1, 1, "Int")),
                    vec![],
                    vec![
                        Expr::String(Box::new(LiteralString { span: span(1, 5, "\"a\""), instance: None })),
                        Expr::Int(Box::new(Int { span: span(1, 10, "5"), instance: None })),
                    ],
                )
            ))
        );
    }

    #[test]
    fn test_generic() {
        assert_eq!(
            new_instance::parse(&generate_tokens(
                r#"
Array[Int, String](5)
           "#
            )),
            Ok((
                &[] as Tokens,
                NewInstance::init(
                    Some(span(1, 1, "Array")),
                    vec![
                        Type::init(Some(span(1, 7, "Int"))),
                        Type::init(Some(span(1, 12, "String"))),
                    ],
                    vec![
                        Expr::Int(Box::new(Int { span: span(1, 20, "5"), instance: None })),
                    ],
                )
            ))
        );
    }
}
