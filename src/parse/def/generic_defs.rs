use parse::{Tokens, ParseResult, tpe, expr};
use parse::tree::GenericDef;
use tokenize::span::Span;
use parse::combinator::{keyword, identifier, symbol, many0, opt, separated_list, capitalize};
use std::cell::Cell;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>
) -> ParseResult<'def, 'r, Vec<GenericDef<'def>>> {
    let (input, _) = symbol('[')(input)?;
    let (input, mut generic_defs) = separated_list(symbol(','), parse_single)(input)?;
    let (input, _) = symbol(']')(input)?;

    for (index, generic_def) in generic_defs.iter_mut().enumerate() {
        generic_def.index = index;
    }

    Ok((input, generic_defs))
}

fn parse_single<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, GenericDef<'def>> {
    let (input, name) = capitalize(input)?;

    Ok((input, GenericDef::init(name, 10000)))
}

#[cfg(test)]
mod tests {
    use parse::Tokens;
    use test_common::{generate_tokens, span};
    use parse::tree::GenericDef;
    use parse::def::generic_defs;

    #[test]
    fn test_simple() {
        assert_eq!(
            generic_defs::parse( &generate_tokens(
                r#"
[A, B]
           "#
            )),
            Ok((
                &[] as Tokens,
                vec![
                    GenericDef::init(span(1, 2, "A"), 0),
                    GenericDef::init(span(1, 5, "B"), 1),
                ]
            ))
        );
    }

    #[test]
    fn test_empty() {
        assert_eq!(
            generic_defs::parse(&generate_tokens(
                r#"
[]
           "#
            )),
            Ok((
                &[] as Tokens,
                vec![]
            ))
        );
    }
}
