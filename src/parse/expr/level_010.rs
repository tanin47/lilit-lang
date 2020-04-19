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

        let (input, expr) = if let Ok((input, invoke)) = invoke::parse_tail(name, input) {
            (
                input,
                Expr::Invoke(Box::new(
                    Invoke {
                        invoker_opt: Some(left),
                        name: invoke.name,
                        args: invoke.args,
                        def_opt: Cell::new(None),
                    }
                ))
            )
        } else {
            (
                input,
                Expr::MemberAccess(Box::new(MemberAccess {
                    parent: left,
                    name: Some(name),
                    def_opt: Cell::new(None)
                }))
            )
        };

        return parse_tail(expr, input);
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
    fn test_dot() {
        assert_eq!(
            level_010::parse(&generate_tokens(
                r#"
func("a").member.another_func()
           "#
            )),
            Ok((
                &[] as Tokens,
                Expr::Invoke(Box::new(
                    Invoke {
                        invoker_opt: Some(Expr::MemberAccess(Box::new(MemberAccess {
                            parent: Expr::Invoke(Box::new(Invoke {
                                invoker_opt: None,
                                name: span(1, 1, "func"),
                                args: vec![Expr::String(Box::new(LiteralString { span: span(1, 6, "\"a\""), instance: RefCell::new(None) }))],
                                def_opt: Cell::new(None),
                            })),
                            name: Some(span(1, 11, "member")),
                            def_opt: Cell::new(None)
                        }))),
                        name: span(1, 18, "another_func"),
                        args: vec![],
                        def_opt: Cell::new(None),
                    }
                ))
            ))
        );
    }
}
