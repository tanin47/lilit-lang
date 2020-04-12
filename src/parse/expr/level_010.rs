use parse::{Tokens, ParseResult, expr};
use parse::tree::{Expr, Invoke, MemberAccess};
use parse::expr::atom;
use parse::combinator::{symbol, separated_list};
use tokenize::token::Token;
use tokenize::span::Span;
use parse::expr::atom::{invoke, identifier};
use std::cell::Cell;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, left) = atom::parse(input)?;

    parse_tail(left, input)
}

fn parse_tail<'def, 'r>(
    left: Expr<'def>,
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Expr<'def>> {
    if let Ok((input, _)) = symbol('.')(input) {
        let (input, name) = identifier::parse_span(input)?;

        return if let Ok((input, invoke)) = invoke::parse_tail(name, input) {
            Ok((
                input,
                Expr::Invoke(Box::new(
                    Invoke {
                        invoker_opt: Some(left),
                        name: invoke.name,
                        args: invoke.args,
                        def_opt: Cell::new(None),
                    }
                ))
            ))
        } else {
            Ok((
                input,
                Expr::MemberAccess(Box::new(MemberAccess {
                    parent: left,
                    name,
                    def_opt: Cell::new(None)
                }))
            ))
        }
    }

    Ok((input, left))
}

#[cfg(test)]
mod tests {
    use parse::Tokens;
    use test_common::{generate_tokens, span};
    use parse::expr::level_010;
    use parse::tree::{Expr, Invoke, LiteralString, MemberAccess, Identifier};
    use std::cell::{Cell, RefCell};

    #[test]
    fn test_chain_method() {
        assert_eq!(
            level_010::parse(&generate_tokens(
                r#"
func("a").another_func()
           "#
            )),
            Ok((
                &[] as Tokens,
                Expr::Invoke(Box::new(
                    Invoke {
                        invoker_opt: Some(Expr::Invoke(Box::new(Invoke {
                            invoker_opt: None,
                            name: span(1, 1, "func"),
                            args: vec![Expr::String(Box::new(LiteralString { span: span(1, 6, "\"a\""), instance: RefCell::new(None) }))],
                            def_opt: Cell::new(None),
                        }))),
                        name: span(1, 11, "another_func"),
                        args: vec![],
                        def_opt: Cell::new(None),
                    }
                ))
            ))
        );
    }

    #[test]
    fn test_member_access() {
        assert_eq!(
            level_010::parse(&generate_tokens(
                r#"
instance.field
           "#
            )),
            Ok((
                &[] as Tokens,
                Expr::MemberAccess(Box::new(MemberAccess {
                    parent: Expr::Identifier(Box::new(Identifier {
                        name: span(1, 1, "instance"),
                        def_opt: Cell::new(None)
                    })),
                    name: span(1, 10, "field"),
                    def_opt: Cell::new(None),
                }))
            ))
        );
    }
}
