use crate::Grammar;
use prytty_core::{Token, TokenKind};

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(input: &str) -> Vec<(TokenKind, &str)> {
        GenericGrammar.tokenize(input).into_iter().map(|t| (t.kind, t.text)).collect()
    }

    fn find(tokens: &[(TokenKind, &str)], text: &str) -> Option<TokenKind> {
        tokens.iter().find(|(_, t)| *t == text).map(|(k, _)| *k)
    }

    #[test]
    fn double_quoted_string() {
        let tokens = tok("\"hello\"");
        assert_eq!(find(&tokens, "\"hello\""), Some(TokenKind::String));
    }

    #[test]
    fn single_quoted_string() {
        let tokens = tok("'world'");
        assert_eq!(find(&tokens, "'world'"), Some(TokenKind::String));
    }

    #[test]
    fn string_with_escape() {
        let tokens = tok("\"say \\\"hi\\\"\"");
        let s = tokens.iter().find(|(k, _)| *k == TokenKind::String);
        assert!(s.is_some(), "expected a String token");
    }

    #[test]
    fn integer() {
        let tokens = tok("42");
        assert_eq!(find(&tokens, "42"), Some(TokenKind::Number));
    }

    #[test]
    fn float_number() {
        let tokens = tok("3.14");
        assert_eq!(find(&tokens, "3.14"), Some(TokenKind::Number));
    }

    #[test]
    fn number_with_underscore() {
        let tokens = tok("1_000_000");
        assert_eq!(find(&tokens, "1_000_000"), Some(TokenKind::Number));
    }

    #[test]
    fn punctuation_chars() {
        for ch in &["(", ")", "{", "}", "[", "]", ":", ";", ",", ".", "=", "+"] {
            let tokens = tok(ch);
            assert_eq!(
                tokens.first().map(|(k, _)| *k),
                Some(TokenKind::Punctuation),
                "'{ch}' should be Punctuation"
            );
        }
    }

    #[test]
    fn plain_word() {
        let tokens = tok("hello");
        assert_eq!(find(&tokens, "hello"), Some(TokenKind::Plain));
    }

    #[test]
    fn plain_word_with_underscore() {
        let tokens = tok("my_var");
        assert_eq!(find(&tokens, "my_var"), Some(TokenKind::Plain));
    }

    #[test]
    fn whitespace_is_plain() {
        let tokens = tok("   ");
        assert!(tokens.iter().all(|(k, _)| *k == TokenKind::Plain));
    }

    #[test]
    fn empty_input_produces_no_tokens() {
        let tokens = tok("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn mixed_input() {
        let tokens = tok("x = 42;");
        assert_eq!(find(&tokens, "x"), Some(TokenKind::Plain));
        assert_eq!(find(&tokens, "="), Some(TokenKind::Punctuation));
        assert_eq!(find(&tokens, "42"), Some(TokenKind::Number));
        assert_eq!(find(&tokens, ";"), Some(TokenKind::Punctuation));
    }
}

pub struct GenericGrammar;

/// Advance pos by one UTF-8 character (1-4 bytes) in a byte slice.
/// Returns the new position, guaranteed to land on a char boundary.
pub(crate) fn advance_char(bytes: &[u8], pos: usize) -> usize {
    if pos >= bytes.len() {
        return pos;
    }
    let b = bytes[pos];
    let char_len = if b < 0x80 {
        1
    } else if b < 0xE0 {
        2
    } else if b < 0xF0 {
        3
    } else {
        4
    };
    (pos + char_len).min(bytes.len())
}

impl Grammar for GenericGrammar {
    fn tokenize<'a>(&self, input: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut pos = 0;
        let bytes = input.as_bytes();

        while pos < bytes.len() {
            // Strings (double or single quoted)
            if bytes[pos] == b'"' || bytes[pos] == b'\'' {
                let quote = bytes[pos];
                let start = pos;
                pos += 1;
                while pos < bytes.len() && bytes[pos] != quote {
                    if bytes[pos] == b'\\' && pos + 1 < bytes.len() {
                        pos += 1; // skip escaped char
                    }
                    pos = advance_char(bytes, pos);
                }
                if pos < bytes.len() {
                    pos += 1; // closing quote
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::String });
                continue;
            }

            // Numbers
            if bytes[pos].is_ascii_digit() {
                let start = pos;
                while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'.' || bytes[pos] == b'_') {
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Number });
                continue;
            }

            // Punctuation/operators
            if b"(){}[]<>:;,.=+-*/%&|!^~?@#".contains(&bytes[pos]) {
                let start = pos;
                pos += 1;
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Punctuation });
                continue;
            }

            // Words (identifiers, keywords — just plain for generic)
            if bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_' {
                let start = pos;
                while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Plain });
                continue;
            }

            // Everything else: whitespace, multi-byte UTF-8, etc.
            let start = pos;
            pos = advance_char(bytes, pos);
            tokens.push(Token { text: &input[start..pos], kind: TokenKind::Plain });
        }

        tokens
    }
}
