use parse::{Tokens, ParseResult, tpe, expr};
use parse::tree::Method;
use tokenize::span::Span;
use parse::combinator::{keyword, identifier, symbol, many0, separated_list, opt};
use parse::def::params;
use std::cell::Cell;

pub fn parse_prefix<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Span<'def>> {
    keyword("def")(input)
}

pub fn parse_tail<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Method<'def>> {
    let (input, name) = identifier(input)?;
    let (input, params) = opt(params::parse)(input)?;
    let (input, _) = symbol(':')(input)?;
    let (input, tpe) = tpe::parse(input)?;

    let (input, exprs) = many0(expr::parse)(input)?;

    let (input, _) = keyword("end")(input)?;

    return Ok((input, Method {
        name,
        params: params.unwrap_or(vec![]),
        exprs,
        return_type: tpe,
        parent_class: None,
        llvm: Cell::new(None)
    }))
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Method<'def>> {
    let (input, _) = parse_prefix(input)?;
    parse_tail(input)
}

#[cfg(test)]
mod tests {
    use parse::Tokens;
    use parse::def::method;
    use test_common::{generate_tokens, span};
    use parse::tree::{Method, Type, Expr, LiteralString, Param};
    use std::cell::{Cell, RefCell};

    #[test]
   fn test_no_params() {
       assert_eq!(
           method::parse(&generate_tokens(
               r#"
def test(): Number
  "hello"
end
           "#
           )),
           Ok((
               &[] as Tokens,
               Method {
                   name: span(1, 5, "test"),
                   params: vec![],
                   exprs: vec![
                       Expr::String(Box::new(LiteralString {
                           span: span(2, 3, "\"hello\""),
                           instance: None
                       }))
                   ],
                   return_type: Type { span: Some(span(1, 13, "Number")), class_def: None },
                   parent_class: None,
                   llvm: Cell::new(None)
               }
           ))
       );
   }

    #[test]
    fn test_with_params() {
        assert_eq!(
            method::parse(&generate_tokens(
                r#"
def test(a: String, b...: String): Number
end
           "#
            )),
            Ok((
                &[] as Tokens,
                Method {
                    name: span(1, 5, "test"),
                    params: vec![
                        Param {
                            name: Some(span(1, 10, "a")),
                            tpe: Type { span: Some(span(1, 13, "String")), class_def: None },
                            is_varargs: false,
                            index: 0,
                            parent: None,
                            llvm: Cell::new(None),
                        },
                        Param {
                            name: Some(span(1, 21, "b")),
                            tpe: Type { span: Some(span(1, 27, "String")), class_def: None },
                            is_varargs: true,
                            index: 1,
                            parent: None,
                            llvm: Cell::new(None),
                        },
                    ],
                    exprs: vec![],
                    return_type: Type { span: Some(span(1, 36, "Number")), class_def: None },
                    parent_class: None,
                    llvm: Cell::new(None)
                }
            ))
        );
    }
}