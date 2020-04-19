use parse::{Tokens, ParseResult, tpe, expr};
use tokenize::span::Span;
use parse::combinator::{keyword, identifier, symbol, many0, capitalize, separated_list, opt};
use parse::tree::Class;
use parse::def::{params, method};
use std::cell::Cell;

pub fn parse_prefix<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Span<'def>> {
    keyword("class")(input)
}

pub fn parse_tail<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Class<'def>> {
    let (input, name) = capitalize(input)?;
    let (input, params) = opt(params::parse)(input)?;

    let (input, methods) = many0(method::parse)(input)?;

    let (input, _) = keyword("end")(input)?;

    return Ok((input, Class {
        name,
        params: params.unwrap_or(vec![]),
        methods,
        llvm: Cell::new(None),
        llvm_native: Cell::new(None),
    }))
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Class<'def>> {
    let (input, _) = parse_prefix(input)?;
    parse_tail(input)
}

#[cfg(test)]
mod tests {
    use parse::Tokens;
    use parse::def::class;
    use test_common::{generate_tokens, span};
    use parse::tree::{Method, Type, Expr, LiteralString, Class};
    use std::cell::Cell;

    #[test]
    fn test_simple() {
        assert_eq!(
            class::parse(&generate_tokens(
                r#"
class Test
  def test: Number
  end
end
           "#
            )),
            Ok((
                &[] as Tokens,
                Class {
                    name: span(1, 7, "Test"),
                    params: vec![],
                    methods: vec![
                        Method {
                            name: span(2, 7, "test"),
                            params: vec![],
                            exprs: vec![],
                            return_type: Type { span: span(2, 13, "Number"), def_opt: Cell::new(None) },
                            llvm: Cell::new(None)
                        }
                    ],
                    llvm: Cell::new(None)
                }
            ))
        );
    }
}
