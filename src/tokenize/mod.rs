use std::io::Write;
use std::ops::Index;
use std::{io, slice};
use tokenize::combinator::{
    concat, take, take_bit, take_hex_number, take_number, take_one_if_case_insensitive, take_while,
};
use tokenize::span::CharAt;
use tokenize::span::Span;
use tokenize::token::Token;
use LilitFile;

pub mod combinator;
pub mod span;
pub mod token;

pub fn apply<'def>(
    content: &'def str,
    file: *const LilitFile<'def>,
) -> Result<Vec<Token<'def>>, Span<'def>> {
    let mut input = Span {
        line: 1,
        col: 1,
        fragment: content,
        file,
    };
    let mut tokens = vec![];

    while input.fragment.len() > 0 {
        let (next_input, token_opt) = tokenize(input)?;
        input = next_input;

        match token_opt {
            Some(Token::Comment(_)) => (),
            Some(token) => tokens.push(token),
            None => (),
        };
    }

    Ok(tokens)
}

fn tokenize<'def>(input: Span<'def>) -> Result<(Span<'def>, Option<Token<'def>>), Span<'def>> {
    let input = skip_space(input);

    if input.fragment.is_empty() {
        return Ok((input, None));
    }

    if let Ok((input, token)) = oneline_comment(input) {
        Ok((input, Some(token)))
    } else if let Ok((input, token)) = string(input) {
        Ok((input, Some(token)))
    } else if let Ok((input, token)) = literal_char(input) {
        Ok((input, Some(token)))
    } else if let Ok((input, token)) = hex(input) {
        Ok((input, Some(token)))
    } else if let Ok((input, token)) = bit(input) {
        Ok((input, Some(token)))
    } else if let Ok((input, token)) = int_or_float(input) {
        Ok((input, Some(token)))
    } else if let Ok((input, token)) = keyword_or_identifier(input) {
        Ok((input, Some(token)))
    } else if let Ok((input, token)) = symbol(input) {
        Ok((input, Some(token)))
    } else {
        Err(input)
    }
}

fn is_whitespace(index: usize, s: &str) -> bool {
    let c = s.char_at(index);
    c == ' ' || c == '\t' || c == '\r' || c == '\n' || c == 12 as char // form feed
}

fn skip_space(input: Span) -> Span {
    let (_, input) = take_while(is_whitespace, input);
    input
}

fn string(input: Span) -> Result<(Span, Token), Span> {
    if input.fragment.char_at(0) != '"' {
        return Err(input);
    }

    let mut escaped = false;
    let mut size = 1;

    for index in 1..input.fragment.len() {
        let c = input.fragment.char_at(index);

        size += 1;

        if escaped {
            escaped = false;
            continue;
        }

        if c == '"' {
            break;
        }

        if c == '\\' {
            escaped = true;
        }
    }

    let (string, after) = take(size, input);

    Ok((after, Token::String(string)))
}

fn literal_char(input: Span) -> Result<(Span, Token), Span> {
    if input.fragment.char_at(0) == '\'' {
    } else {
        return Err(input);
    }

    let (literal_char, after) = take_while(
        |index, s| {
            let end_cond =
                index >= 2 && s.char_at(index - 2) != '\\' && s.char_at(index - 1) == '\'';
            let end_cond_2 = index >= 3
                && s.char_at(index - 3) == '\\'
                && s.char_at(index - 2) == '\\'
                && s.char_at(index - 1) == '\'';

            !(end_cond || end_cond_2)
        },
        input,
    );

    Ok((after, Token::Char(literal_char)))
}

fn hex_p<'a>(num: Span<'a>, original: Span<'a>) -> Result<(Span<'a>, Token<'a>), Span<'a>> {
    let (must_be_p, input) = take_one_if_case_insensitive("P", original);

    if must_be_p.fragment.is_empty() {
        return Err(original);
    }

    let (maybe_sign, input) = take_one_if_case_insensitive("-+", input);
    let (exponent, input) = take_number(input);

    if exponent.fragment.is_empty() {
        return Err(input);
    }

    let num = concat(&[num, must_be_p, maybe_sign, exponent]);

    Ok((input, Token::Float(num)))
}

fn hex_decimal<'a>(num: Span<'a>, original: Span<'a>) -> Result<(Span<'a>, Token<'a>), Span<'a>> {
    let (maybe_p_or_dot, input) = take_one_if_case_insensitive("P.", original);

    if maybe_p_or_dot.fragment.is_empty() {
        return Ok((input, Token::Int(num)));
    }

    if maybe_p_or_dot.fragment.char_at(0) != '.' {
        return hex_p(num, original);
    }

    let must_be_dot = maybe_p_or_dot;
    assert_eq!(must_be_dot.fragment, ".");

    let (decimal, input) = take_hex_number(input);

    if num.fragment.is_empty() && decimal.fragment.is_empty() {
        return Err(original);
    }

    let num = concat(&[num, must_be_dot, decimal]);

    hex_p(num, input)
}

fn hex(original: Span) -> Result<(Span, Token), Span> {
    if original.fragment.len() >= 2
        && original.fragment.char_at(0) == '0'
        && original.fragment.char_at(1).to_ascii_uppercase() == 'X'
    {
    } else {
        return Err(original);
    }

    let (prefix, input) = take(2, original);
    let (hex, input) = take_hex_number(input);

    let num = concat(&[prefix, hex]);

    hex_decimal(num, input)
}

fn bit(original: Span) -> Result<(Span, Token), Span> {
    if original.fragment.len() >= 2
        && original.fragment.char_at(0) == '0'
        && original.fragment.char_at(1).to_ascii_uppercase() == 'B'
    {
    } else {
        return Err(original);
    }

    let (prefix, input) = take(2, original);
    let (bits, input) = take_bit(input);

    let num = concat(&[prefix, bits]);

    Ok((input, Token::Int(num)))
}

fn is_identifier(index: usize, s: &str) -> bool {
    let c = s.char_at(index);
    c >= '0' && c <= '9' || c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '_' || c == '$'
}

fn is_keyword(s: &str) -> bool {
    match s {
        "class" | "def" | "static" | "end" | "while" => true,
        _ => false,
    }
}

fn keyword_or_identifier(original: Span) -> Result<(Span, Token), Span> {
    let (ident, input) = take_while(is_identifier, original);

    if ident.fragment.is_empty() {
        return Err(original);
    }

    if is_keyword(ident.fragment) {
        Ok((input, Token::Keyword(ident)))
    } else if ident.fragment.char_at(0) == ident.fragment.char_at(0).to_uppercase().to_string().char_at(0) {
        Ok((input, Token::Capitalize(ident)))
    } else {
        Ok((input, Token::Identifier(ident)))
    }
}

fn symbol(input: Span) -> Result<(Span, Token), Span> {
    // No need to check anything before word(..) and is_whitespace(..) guarantees that this is a symbol.

    let (symbol, input) = take(1, input);

    Ok((input, Token::Symbol(symbol)))
}

fn float_e<'a>(
    num: Span<'a>,
    original: Span<'a>,
    include_dot: bool,
) -> Result<(Span<'a>, Token<'a>), Span<'a>> {
    let (maybe_e, input) = take_one_if_case_insensitive("E", original);

    if maybe_e.fragment.is_empty() {
        if include_dot {
            return Ok((original, Token::Float(num)))
        } else {
            return Err(input);
        }
    }

    if num.fragment.is_empty() {
        return Err(input);
    }

    let (maybe_operator, input) = take_one_if_case_insensitive("+-", input);
    let (second_number, input) = take_number(input);

    let num = concat(&[num, maybe_e, maybe_operator, second_number]);

    Ok((input, Token::Float(num)))
}

fn float_dot<'a>(
    first_number: Span<'a>,
    original: Span<'a>,
) -> Result<(Span<'a>, Token<'a>), Span<'a>> {
    let (symbol, input) = take(1, original);
    let last_char = if !symbol.fragment.is_empty() {
        symbol.fragment.char_at(0).to_ascii_uppercase()
    } else {
        ' ' // anything that is not a dot.
    };

    if last_char != '.' {
        return float_e(first_number, original, false);
    }

    let (second_number, input) = take_number(input);

    if second_number.fragment.is_empty() {
        return Err(original);
    }

    let num = Span {
        line: first_number.line,
        col: first_number.col,
        fragment: unsafe {
            std::str::from_utf8_unchecked(slice::from_raw_parts(
                first_number.fragment.as_ptr(),
                first_number.fragment.len() + symbol.fragment.len() + second_number.fragment.len(),
            ))
        },
        file: first_number.file,
    };

    float_e(num, input, true)
}

fn int_or_float(original: Span) -> Result<(Span, Token), Span> {
    let (number, input) = take_number(original);

    if let Ok(ok) = float_dot(number, input) {
        return Ok(ok);
    }

    if number.fragment.is_empty() {
        return Err(original);
    }

    Ok((input, Token::Int(number)))
}

fn oneline_comment(input: Span) -> Result<(Span, Token), Span> {
    if input.fragment.len() >= 2
        && input.fragment.char_at(0) == '/'
        && input.fragment.char_at(1) == '/'
    {
    } else {
        return Err(input);
    }

    let (comment, after) = take_while(|index, s| s.char_at(index) != '\n', input);

    Ok((after, Token::Comment(comment)))
}

#[cfg(test)]
mod tests {
    use test_common::{generate_tokens, span};
    use tokenize::span::Span;
    use tokenize::token::Token;

    fn apply(content: &str) -> Result<Vec<Token>, Span> {
        super::apply(content, std::ptr::null())
    }

    #[test]
    fn test_oneline_comment() {
        assert_eq!(apply("// test"), Ok(vec![]))
    }

    #[test]
    fn test_unicode() {
        assert_eq!(
            apply(
                r#"
"打包选项"
"#
                    .trim()
            ),
            Ok(vec![Token::String(span(1, 1, "\"打包选项\""))])
        )
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(
            apply(
                r#"
"" +
"#
                    .trim()
            ),
            Ok(vec![
                Token::String(span(1, 1, "\"\"")),
                Token::Symbol(span(1, 4, "+"))
            ])
        )
    }

    #[test]
    fn test_string() {
        assert_eq!(
            apply(
                r#"
"ab\"c" +
"#
                    .trim()
            ),
            Ok(vec![
                Token::String(span(1, 1, "\"ab\\\"c\"")),
                Token::Symbol(span(1, 9, "+"))
            ])
        )
    }

    #[test]
    fn test_escaped_backslash_string() {
        assert_eq!(
            apply(
                r#"
"\"" +
"#
                    .trim()
            ),
            Ok(vec![
                Token::String(span(1, 1, "\"\\\"\"")),
                Token::Symbol(span(1, 6, "+"))
            ])
        )
    }

    #[test]
    fn test_escaped_backslash_string_2() {
        assert_eq!(
            apply(
                r#"
"\\" +
"#
                    .trim()
            ),
            Ok(vec![
                Token::String(span(1, 1, "\"\\\\\"")),
                Token::Symbol(span(1, 6, "+"))
            ])
        )
    }

    #[test]
    fn test_escaped_backslash_string_3() {
        assert_eq!(
            apply(
                r#"
"\\\"" +
"#
                    .trim()
            ),
            Ok(vec![
                Token::String(span(1, 1, "\"\\\\\\\"\"")),
                Token::Symbol(span(1, 8, "+"))
            ])
        )
    }

    #[test]
    fn test_empty_char() {
        assert_eq!(
            apply(
                r#"
'' +
"#
                    .trim()
            ),
            Ok(vec![
                Token::Char(span(1, 1, "''")),
                Token::Symbol(span(1, 4, "+"))
            ])
        )
    }

    #[test]
    fn test_char() {
        assert_eq!(
            apply(
                r#"
'a' +
"#
                    .trim()
            ),
            Ok(vec![
                Token::Char(span(1, 1, "'a'")),
                Token::Symbol(span(1, 5, "+"))
            ])
        )
    }

    #[test]
    fn test_escaped_char() {
        assert_eq!(
            apply(
                r#"
'\'' +
"#
                    .trim()
            ),
            Ok(vec![
                Token::Char(span(1, 1, "'\\''")),
                Token::Symbol(span(1, 6, "+"))
            ])
        )
    }

    #[test]
    fn test_escaped_backslash_char() {
        assert_eq!(
            apply(
                r#"
'\\' +
"#
                    .trim()
            ),
            Ok(vec![
                Token::Char(span(1, 1, "'\\\\'")),
                Token::Symbol(span(1, 6, "+"))
            ])
        )
    }

    #[test]
    fn test_bit() {
        assert_eq!(
            apply(
                r#"
0b001
"#
                    .trim()
            ),
            Ok(vec![
                Token::Int(span(1, 1, "0b001")),
            ])
        )
    }

    #[test]
    fn test_hex() {
        assert_eq!(
            apply(
                r#"
0x0123456789abcdefABCDEF 0x02 0x2p+3 0x2p-3 0xap2 0x1.0p2 0x.1p2 0x.ap2
"#
                    .trim()
            ),
            Ok(vec![
                Token::Int(span(1, 1, "0x0123456789abcdefABCDEF")),
                Token::Int(span(1, 26, "0x02")),
                Token::Float(span(1, 31, "0x2p+3")),
                Token::Float(span(1, 38, "0x2p-3")),
                Token::Float(span(1, 45, "0xap2")),
                Token::Float(span(1, 51, "0x1.0p2")),
                Token::Float(span(1, 59, "0x.1p2")),
                Token::Float(span(1, 66, "0x.ap2")),
            ])
        )
    }

    #[test]
    fn test_int() {
        assert_eq!(
            apply(
                r#"
1234567890
"#
                    .trim()
            ),
            Ok(vec![Token::Int(span(1, 1, "1234567890"))])
        )
    }

    #[test]
    fn test_float() {
        assert_eq!(
            apply(
                r#"
1.0 1e2 1.3e-2 12e+1
"#
                    .trim()
            ),
            Ok(vec![
                Token::Float(span(1, 1, "1.0")),
                Token::Float(span(1, 5, "1e2")),
                Token::Float(span(1, 9, "1.3e-2")),
                Token::Float(span(1, 16, "12e+1")),
            ])
        )
    }

    #[test]
    fn test_word() {
        assert_eq!(
            apply(
                r#"
a_b$1B
"#
                    .trim()
            ),
            Ok(vec![Token::Identifier(span(1, 1, "a_b$1B"))])
        )
    }

    #[test]
    fn test_symbol() {
        assert_eq!(
            apply(
                r#"
>>
"#
                    .trim()
            ),
            Ok(vec![
                Token::Symbol(span(1, 1, ">")),
                Token::Symbol(span(1, 2, ">"))
            ])
        )
    }

    #[test]
    fn test_complex() {
        assert_eq!(
            apply(
                r#"
def method(): Boolean
    // test
    a += 1
end
"#
                    .trim()
            ),
            Ok(vec![
                Token::Keyword(span(1, 1, "def")),
                Token::Identifier(span(1, 5, "method")),
                Token::Symbol(span(1, 11, "(")),
                Token::Symbol(span(1, 12, ")")),
                Token::Symbol(span(1, 13, ":")),
                Token::Capitalize(span(1, 15, "Boolean")),
                Token::Identifier(span(3, 5, "a")),
                Token::Symbol(span(3, 7, "+")),
                Token::Symbol(span(3, 8, "=")),
                Token::Int(span(3, 10, "1")),
                Token::Keyword(span(4, 1, "end")),
            ])
        )
    }
}