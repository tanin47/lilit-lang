use parse::{Tokens, ParseResult, tpe, expr};
use parse::tree::{Method, Param, ParamParent};
use tokenize::span::Span;
use parse::combinator::{keyword, identifier, symbol, many0, opt, separated_list};
use std::cell::Cell;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>
) -> ParseResult<'def, 'r, Vec<Param<'def>>> {
    let (input, _) = symbol('(')(input)?;
    let (input, params) = separated_list(symbol(','), parse_single)(input)?;
    let (input, _) = symbol(')')(input)?;

    Ok((input, params))
}

fn parse_single<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Param<'def>> {
    let (input, name) = identifier(input)?;
    let (input, varargs_opt) = opt(parse_varargs)(input)?;
    let (input, _) = symbol(':')(input)?;
    let (input, tpe) = tpe::parse(input)?;

    Ok((input, Param {
        name: Some(name),
        tpe,
        is_varargs: varargs_opt.is_some(),
        index: 100000,
        parent: Cell::new(None),
        llvm: Cell::new(None),
    }))
}

pub fn parse_varargs<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, ()> {
    let (input, _) = symbol('.')(input)?;
    let (input, _) = symbol('.')(input)?;
    let (input, _) = symbol('.')(input)?;
    Ok((input, ()))
}

#[cfg(test)]
mod tests {
    use parse::Tokens;
    use parse::def::params;
    use test_common::{generate_tokens, span};
    use parse::tree::{Method, Type, Expr, LiteralString, Param, ParamParent};
    use std::cell::Cell;

    #[test]
    fn test_simple() {
        assert_eq!(
            params::parse( &generate_tokens(
                r#"
(arg: Number, arg2...: Number)
           "#
            )),
            Ok((
                &[] as Tokens,
                vec![
                    Param {
                        name: Some(span(1, 2, "arg")),
                        tpe: Type { span: Some(span(1, 7, "Number")), def_opt: Cell::new(None) },
                        is_varargs: false,
                        index: 0,
                        parent: Cell::new(None),
                        llvm: Cell::new(None),
                    },
                    Param {
                        name: Some(span(1, 15, "arg2")),
                        tpe: Type { span: Some(span(1, 24, "Number")), def_opt: Cell::new(None) },
                        is_varargs: true,
                        index: 1,
                        parent: Cell::new(None),
                        llvm: Cell::new(None),
                    }
                ]
            ))
        );
    }

    #[test]
    fn test_empty() {
        assert_eq!(
            params::parse(&generate_tokens(
                r#"
()
           "#
            )),
            Ok((
                &[] as Tokens,
                vec![]
            ))
        );
    }
}
