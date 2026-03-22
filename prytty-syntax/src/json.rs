use crate::generic::advance_char;
use crate::Grammar;
use prytty_core::{Token, TokenKind};

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(input: &str) -> Vec<(TokenKind, &str)> {
        JsonGrammar.tokenize(input).into_iter().map(|t| (t.kind, t.text)).collect()
    }

    fn find(tokens: &[(TokenKind, &str)], text: &str) -> Option<TokenKind> {
        tokens.iter().find(|(_, t)| *t == text).map(|(k, _)| *k)
    }

    #[test]
    fn key_is_key_kind() {
        let tokens = tok(r#"{"name": "alice"}"#);
        assert_eq!(find(&tokens, "\"name\""), Some(TokenKind::Key));
    }

    #[test]
    fn string_value_is_string_kind() {
        let tokens = tok(r#"{"name": "alice"}"#);
        assert_eq!(find(&tokens, "\"alice\""), Some(TokenKind::String));
    }

    #[test]
    fn integer_value() {
        let tokens = tok(r#"{"age": 42}"#);
        assert_eq!(find(&tokens, "42"), Some(TokenKind::Number));
    }

    #[test]
    fn negative_number() {
        let tokens = tok(r#"{"x": -7}"#);
        assert_eq!(find(&tokens, "-7"), Some(TokenKind::Number));
    }

    #[test]
    fn float_value() {
        let tokens = tok(r#"{"pi": 3.14}"#);
        assert_eq!(find(&tokens, "3.14"), Some(TokenKind::Number));
    }

    #[test]
    fn constant_true() {
        let tokens = tok(r#"{"ok": true}"#);
        assert_eq!(find(&tokens, "true"), Some(TokenKind::Constant));
    }

    #[test]
    fn constant_false() {
        let tokens = tok(r#"{"ok": false}"#);
        assert_eq!(find(&tokens, "false"), Some(TokenKind::Constant));
    }

    #[test]
    fn constant_null() {
        let tokens = tok(r#"{"x": null}"#);
        assert_eq!(find(&tokens, "null"), Some(TokenKind::Constant));
    }

    #[test]
    fn braces_are_punctuation() {
        let tokens = tok("{}");
        for (k, t) in &tokens {
            if *t == "{" || *t == "}" {
                assert_eq!(*k, TokenKind::Punctuation, "'{t}' should be Punctuation");
            }
        }
    }

    #[test]
    fn colon_is_punctuation() {
        let tokens = tok(r#"{"a": 1}"#);
        assert_eq!(find(&tokens, ":"), Some(TokenKind::Punctuation));
    }

    #[test]
    fn comma_is_punctuation() {
        let tokens = tok(r#"{"a": 1, "b": 2}"#);
        assert_eq!(find(&tokens, ","), Some(TokenKind::Punctuation));
    }

    #[test]
    fn nested_object_inner_key() {
        let tokens = tok(r#"{"outer": {"inner": 99}}"#);
        assert_eq!(find(&tokens, "\"outer\""), Some(TokenKind::Key));
        assert_eq!(find(&tokens, "\"inner\""), Some(TokenKind::Key));
        assert_eq!(find(&tokens, "99"), Some(TokenKind::Number));
    }

    #[test]
    fn array_values_are_strings() {
        let tokens = tok(r#"["a", "b"]"#);
        // Neither "a" nor "b" is followed by a colon, so they're String kind
        assert_eq!(find(&tokens, "\"a\""), Some(TokenKind::String));
        assert_eq!(find(&tokens, "\"b\""), Some(TokenKind::String));
    }

    #[test]
    fn string_with_escape() {
        let tokens = tok(r#"{"msg": "hello\"world"}"#);
        let s = tokens.iter().find(|(k, t)| *k == TokenKind::String && t.contains("hello"));
        assert!(s.is_some(), "expected a String token containing escaped quote");
    }

    #[test]
    fn empty_input_produces_no_tokens() {
        let tokens = tok("");
        assert!(tokens.is_empty());
    }
}

pub struct JsonGrammar;

impl Grammar for JsonGrammar {
    fn tokenize<'a>(&self, input: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut pos = 0;
        let bytes = input.as_bytes();
        let mut _expect_key = true;

        while pos < bytes.len() {
            // Whitespace
            if bytes[pos].is_ascii_whitespace() {
                let start = pos;
                while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Plain });
                continue;
            }

            // Strings
            if bytes[pos] == b'"' {
                let start = pos;
                pos += 1;
                while pos < bytes.len() && bytes[pos] != b'"' {
                    if bytes[pos] == b'\\' && pos + 1 < bytes.len() {
                        pos += 1;
                    }
                    pos += 1;
                }
                if pos < bytes.len() {
                    pos += 1;
                }
                // If next non-ws char is ':', this is a key
                let kind = if is_key(bytes, pos) {
                    TokenKind::Key
                } else {
                    TokenKind::String
                };
                tokens.push(Token { text: &input[start..pos], kind });
                _expect_key = false;
                continue;
            }

            // Numbers
            if bytes[pos].is_ascii_digit() || (bytes[pos] == b'-' && pos + 1 < bytes.len() && bytes[pos + 1].is_ascii_digit()) {
                let start = pos;
                if bytes[pos] == b'-' {
                    pos += 1;
                }
                while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                    pos += 1;
                }
                if pos < bytes.len() && bytes[pos] == b'.' {
                    pos += 1;
                    while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                        pos += 1;
                    }
                }
                if pos < bytes.len() && (bytes[pos] == b'e' || bytes[pos] == b'E') {
                    pos += 1;
                    if pos < bytes.len() && (bytes[pos] == b'+' || bytes[pos] == b'-') {
                        pos += 1;
                    }
                    while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                        pos += 1;
                    }
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Number });
                continue;
            }

            // Keywords: true, false, null
            if pos + 4 <= bytes.len() && &input[pos..pos + 4] == "true" && !is_ident_char(bytes, pos + 4) {
                tokens.push(Token { text: &input[pos..pos + 4], kind: TokenKind::Constant });
                pos += 4;
                continue;
            }
            if pos + 5 <= bytes.len() && &input[pos..pos + 5] == "false" && !is_ident_char(bytes, pos + 5) {
                tokens.push(Token { text: &input[pos..pos + 5], kind: TokenKind::Constant });
                pos += 5;
                continue;
            }
            if pos + 4 <= bytes.len() && &input[pos..pos + 4] == "null" && !is_ident_char(bytes, pos + 4) {
                tokens.push(Token { text: &input[pos..pos + 4], kind: TokenKind::Constant });
                pos += 4;
                continue;
            }

            // Structural: { } [ ] , :
            if b"{}[],:".contains(&bytes[pos]) {
                let ch = bytes[pos];
                let start = pos;
                pos += 1;
                let kind = TokenKind::Punctuation;
                tokens.push(Token { text: &input[start..pos], kind });
                _expect_key = ch == b'{' || ch == b',';
                continue;
            }

            // Anything else (including multi-byte UTF-8)
            let start = pos;
            pos = advance_char(bytes, pos);
            tokens.push(Token { text: &input[start..pos], kind: TokenKind::Plain });
        }

        tokens
    }
}

fn is_key(bytes: &[u8], pos: usize) -> bool {
    let mut i = pos;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i < bytes.len() && bytes[i] == b':'
}

fn is_ident_char(bytes: &[u8], pos: usize) -> bool {
    pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_')
}
