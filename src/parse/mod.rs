use std::borrow::Borrow;
use std::ops::Deref;
use std::pin::Pin;
use std::ptr::null;

use {LilitFile, tokenize};
use parse::tree::CompilationUnit;
use tokenize::span::Span;
use tokenize::token::Token;

pub mod combinator;
pub mod compilation_unit;
pub mod def;
pub mod expr;
pub mod tpe;
pub mod tree;

pub type Tokens<'def, 'r> = &'r [Token<'def>];
pub type ParseResult<'def, 'r, T> = Result<(Tokens<'def, 'r>, T), Tokens<'def, 'r>>;

pub fn apply_tokens<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> Result<CompilationUnit<'def>, Tokens<'def, 'r>> {
    let result = compilation_unit::parse(input);

    match result {
        Ok((err_input, unit)) => {
            if err_input.is_empty() {
                Ok(unit)
            } else {
                Err(err_input)
            }
        }
        Err(e) => Err(e),
    }
}

pub fn apply<'def, 'input, 'path>(
    input: &'input str,
    path: &'path str,
) -> Result<Pin<Box<LilitFile<'def>>>, Span<'def>> {
    let mut file = Pin::new(Box::new(LilitFile {
        unit: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
        content: input.to_owned(),
        path: path.to_owned(),
    }));
    let tokens = match tokenize::apply(unsafe { &*(file.content.as_ref() as *const str) }, &*file) {
        Ok(tokens) => tokens,
        Err(span) => return Err(span),
    };
    let unit = match apply_tokens(
        unsafe { &*(&tokens as *const Vec<Token<'def>>) },
    ) {
        Ok(unit) => unit,
        Err(tokens) => return Err(tokens.first().unwrap().span()),
    };

    file.unit = unit;

    Ok(file)
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use std::pin::Pin;

    use LilitFile;
    use parse::{apply, Tokens};
    use parse::tree::{Class, CompilationUnit, CompilationUnitItem, Method, Type};
    use test_common::{generate_tokens, span, span2};
    use std::cell::Cell;

    #[test]
    fn test_simple() {
        let content = r#"
class Test
  def test(): Number
  end
end
                           "#.trim();
        let path = "/path.lilit";
        let file = unwrap!(Ok, apply(content, path));
        assert_eq!(
            file,
            Pin::new(Box::new(LilitFile {
                unit: CompilationUnit {
                    items: vec![
                        CompilationUnitItem::Class(
                            Class {
                                name: span2(1, 7, "Test", file.deref()),
                                generics: vec![],
                                params: vec![],
                                methods: vec![
                                    Method {
                                        name: span2(2, 7, "test", file.deref()),
                                        params: vec![],
                                        exprs: vec![],
                                        return_type: Type::init(Some(span2(2, 15, "Number", file.deref()))),
                                        parent_class: None,
                                        llvm: Cell::new(None),
                                    }
                                ],
                                llvm: Cell::new(None),
                                llvm_native: Cell::new(None)
                            }
                        )
                    ]
                },
                content: content.to_string(),
                path: path.to_string(),
            }))
        );
    }
}
