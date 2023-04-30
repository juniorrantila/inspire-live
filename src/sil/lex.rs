#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Token {
    Number(&'static str),
    Text(&'static str),
    Quoted(&'static str),

    OpenBracket(&'static str),
    CloseBracket(&'static str),

    Colon(&'static str),
    EqualSign(&'static str),
}

// NOTE: These are only meant to be used in tests
#[cfg(test)]
impl Token {
    pub fn open_bracket() -> Self {
        Self::OpenBracket("[")
    }
    pub fn close_bracket() -> Self {
        Self::CloseBracket("]")
    }
    pub fn colon() -> Self {
        Self::Colon(":")
    }
    pub fn equal_sign() -> Self {
        Self::EqualSign("=")
    }
}

impl Token {
    pub fn raw_text(&self) -> &'static str {
        match self {
            Token::Number(text) => text,
            Token::Text(text) => text,
            Token::Quoted(text) => text,
            Token::OpenBracket(text) => text,
            Token::CloseBracket(text) => text,
            Token::Colon(text) => text,
            Token::EqualSign(text) => text,
        }
    }

    pub fn text(&self) -> &'static str {
        match self {
            Token::Quoted(text) => remove_quotes(text),
            _ => self.text(),
        }
    }
}

fn remove_quotes(quoted: &'static str) -> &'static str {
    if quoted.len() < 2 {
        return &quoted[0..0];
    }
    if quoted.ends_with("\"") {
        return &quoted[1..quoted.len() - 1];
    }
    return &quoted[1..];
}

pub fn lex(mut content: &'static str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    while !content.is_empty() {
        match content.as_bytes() {
            [b'0'..=b'9', ..] => {
                let res = lex_number(content);
                tokens.push(Token::Number(res));
                content = &content[res.len()..];
            }

            [b' ' | b'\n', ..] => {
                content = &content[1..];
            }

            [b'"', ..] => {
                let res = lex_quoted(content);
                tokens.push(Token::Quoted(res));
                content = &content[res.len()..];
            }

            [b'[', ..] => {
                tokens.push(Token::OpenBracket(&content[..1]));
                content = &content[1..];
            }

            [b']', ..] => {
                tokens.push(Token::CloseBracket(&content[..1]));
                content = &content[1..];
            }

            [b':', ..] => {
                tokens.push(Token::Colon(&content[..1]));
                content = &content[1..];
            }

            [b'=', ..] => {
                tokens.push(Token::EqualSign(&content[..1]));
                content = &content[1..];
            }

            [_] | [_, ..] => {
                let res = lex_text(content);
                tokens.push(Token::Text(remove_trailing_new_lines(res)));
                content = &content[res.len()..];
            }

            [] => break,
        };
    }

    return tokens;
}

fn lex_text(content: &str) -> &str {
    let mut end_index = 0;

    let mut bytes = content.as_bytes();
    while !bytes.is_empty() {
        match bytes {
            [b'=', ..] | [b':', ..] | [b'[', ..] | [b']', ..] => break,
            _ => {
                end_index += 1;
                bytes = &bytes[1..];
            }
        };
    }

    return &content[..end_index];
}

fn lex_number(content: &str) -> &str {
    let mut end_index = 0;

    let mut chars = content.chars();
    while let Some(c) = chars.next() {
        match c {
            '0'..='9' | '.' => {
                end_index += 1;
            }
            _ => {
                break;
            }
        };
    }

    return &content[..end_index];
}

// FIXME: Handle escapes.
fn lex_quoted(content: &str) -> &str {
    let quoted = &content[1..]; // Remove starting quote.

    let mut end_index = 1;

    let mut bytes = quoted.as_bytes();
    while !bytes.is_empty() {
        match bytes {
            [b'"', ..] => {
                end_index += 1;
                break;
            }
            [_, ..] => {
                end_index += 1;
                bytes = &bytes[1..];
            }
            [] => break,
        };
    }

    return &content[..end_index];
}

pub fn clean_up_for_attribute_key(content: &str) -> &str {
    let mut trailing_spaces = 0;
    let chars = content.chars();
    for c in chars.rev() {
        if c == ' ' {
            trailing_spaces += 1;
            continue;
        }
        break;
    }
    return &content[..content.len() - trailing_spaces];
}

fn remove_trailing_new_lines(content: &str) -> &str {
    let mut trailing_new_lines = 0;
    let chars = content.chars();
    for c in chars.rev() {
        if c == '\n' {
            trailing_new_lines += 1;
            continue;
        }
        break;
    }
    return &content[..content.len() - trailing_new_lines];
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn can_lex_simple() {
        let input = "[title]\nFoobar\n\n[text]\ncolor = 13.37 19.84 42\n\nHello there :)";
        let tokens = lex(input);
        assert_eq!(
            tokens,
            [
                Token::OpenBracket("["),
                Token::Text("title"),
                Token::CloseBracket("]"),
                Token::Text("Foobar"),
                Token::OpenBracket("["),
                Token::Text("text"),
                Token::CloseBracket("]"),
                Token::Text("color "),
                Token::EqualSign("="),
                Token::Number("13.37"),
                Token::Number("19.84"),
                Token::Number("42"),
                Token::Text("Hello there "),
                Token::Colon(":"),
                Token::Text(")")
            ]
        );
    }

    #[test]
    fn can_lex_quoted() {
        assert_eq!(lex("\"black\""), [Token::Quoted("\"black\"")]);
    }

    #[test]
    fn can_lex_incomplete_quoted() {
        assert_eq!(lex("\"black"), [Token::Quoted("\"black")]);
    }

    #[test]
    fn can_lex_non_ascii() {
        let input = "[title]\nFööbär\n\n[text]\ncölår = 13.37 19.84 42\n\nHallå där :)";
        let tokens = lex(input);
        assert_eq!(
            tokens,
            [
                Token::open_bracket(),
                Token::Text("title"),
                Token::close_bracket(),
                Token::Text("Fööbär"),
                Token::open_bracket(),
                Token::Text("text"),
                Token::close_bracket(),
                Token::Text("cölår "),
                Token::equal_sign(),
                Token::Number("13.37"),
                Token::Number("19.84"),
                Token::Number("42"),
                Token::Text("Hallå där "),
                Token::colon(),
                Token::Text(")")
            ]
        );
    }

    #[test]
    fn can_lex_without_new_line() {
        let input = "[title]Foobar[text]color = 13.37 19.84 42 Hello there :)";
        let tokens = lex(input);
        assert_eq!(
            tokens,
            [
                Token::open_bracket(),
                Token::Text("title"),
                Token::close_bracket(),
                Token::Text("Foobar"),
                Token::open_bracket(),
                Token::Text("text"),
                Token::close_bracket(),
                Token::Text("color "),
                Token::equal_sign(),
                Token::Number("13.37"),
                Token::Number("19.84"),
                Token::Number("42"),
                Token::Text("Hello there "),
                Token::colon(),
                Token::Text(")")
            ]
        );
    }

    #[test]
    fn can_lex_with_new_lines() {
        let input = "[title]

Foobar

[text] color = 13.37 19.84 42

Hello there :)
";
        let tokens = lex(input);
        assert_eq!(
            tokens,
            [
                Token::open_bracket(),
                Token::Text("title"),
                Token::close_bracket(),
                Token::Text("Foobar"),
                Token::open_bracket(),
                Token::Text("text"),
                Token::close_bracket(),
                Token::Text("color "),
                Token::equal_sign(),
                Token::Number("13.37"),
                Token::Number("19.84"),
                Token::Number("42"),
                Token::Text("Hello there "),
                Token::colon(),
                Token::Text(")")
            ]
        );
    }

    #[test]
    fn ignores_leading_and_trailing_new_lines_in_bodies() {
        let input = "[title]

This text is the same node

as this text.

[text] color = 13.37 19.84 42

Hello there :)
";
        let tokens = lex(input);
        assert_eq!(
            tokens,
            [
                Token::open_bracket(),
                Token::Text("title"),
                Token::close_bracket(),
                Token::Text("This text is the same node\n\nas this text."),
                Token::open_bracket(),
                Token::Text("text"),
                Token::close_bracket(),
                Token::Text("color "),
                Token::equal_sign(),
                Token::Number("13.37"),
                Token::Number("19.84"),
                Token::Number("42"),
                Token::Text("Hello there "),
                Token::colon(),
                Token::Text(")")
            ]
        );
    }
}
