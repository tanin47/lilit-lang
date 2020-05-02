use parse::{Tokens, ParseResult, tpe, expr};
use tokenize::span::Span;
use parse::combinator::{keyword, identifier, symbol, many0, capitalize, separated_list, opt};
use parse::tree::Class;
use parse::def::{params, method, generic_defs};
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
    let (input, generics) = opt(generic_defs::parse)(input)?;
    let (input, params) = opt(params::parse)(input)?;

    let (input, methods) = many0(method::parse)(input)?;

    let (input, _) = keyword("end")(input)?;

    return Ok((input, Class {
        name,
        generics: generics.unwrap_or(vec![]),
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
    use parse::tree::{Method, Type, Expr, LiteralString, Class, Param, GenericDef};
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
                    generics: vec![],
                    params: vec![],
                    methods: vec![
                        Method {
                            name: span(2, 7, "test"),
                            params: vec![],
                            exprs: vec![],
                            return_type: Type::init(Some(span(2, 13, "Number"))),
                            parent_class: None,
                            llvm: Cell::new(None)
                        }
                    ],
                    llvm: Cell::new(None),
                    llvm_native: Cell::new(None),
                }
            ))
        );
    }

    #[test]
    fn test_complex() {
        assert_eq!(
            class::parse(&generate_tokens(
                r#"
class Test[T](a: Number)
  def test: T
  end
end
           "#
            )),
            Ok((
                &[] as Tokens,
                Class {
                    name: span(1, 7, "Test"),
                    generics: vec![
                        GenericDef::init(span(1, 12, "T"), 0)
                    ],
                    params: vec![
                      Param::init(
                          Some(span(1, 15, "a")),
                          Type::init(Some(span(1, 18, "Number"))),
                          false,
                          0
                      )
                    ],
                    methods: vec![
                        Method {
                            name: span(2, 7, "test"),
                            params: vec![],
                            exprs: vec![],
                            return_type: Type::init(Some(span(2, 13, "T"))),
                            parent_class: None,
                            llvm: Cell::new(None)
                        }
                    ],
                    llvm: Cell::new(None),
                    llvm_native: Cell::new(None),
                }
            ))
        );
    }
}
