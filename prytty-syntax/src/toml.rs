use crate::Grammar;
use prytty_core::{Token, TokenKind};

pub struct TomlGrammar;

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(input: &str) -> Vec<(TokenKind, &str)> {
        TomlGrammar.tokenize(input).into_iter().map(|t| (t.kind, t.text)).collect()
    }

    fn find(tokens: &[(TokenKind, &str)], text: &str) -> Option<TokenKind> {
        tokens.iter().find(|(_, t)| *t == text).map(|(k, _)| *k)
    }

    #[test]
    fn section_header() {
        let tokens = tok("[package]");
        assert_eq!(find(&tokens, "[package]"), Some(TokenKind::Attribute));
    }

    #[test]
    fn array_section_header() {
        let tokens = tok("[[dependencies]]");
        assert_eq!(find(&tokens, "[[dependencies]]"), Some(TokenKind::Attribute));
    }

    #[test]
    fn comment_line() {
        let tokens = tok("# This is a comment");
        assert_eq!(find(&tokens, "# This is a comment"), Some(TokenKind::Comment));
    }

    #[test]
    fn key_is_key_kind() {
        let tokens = tok("name = \"prytty\"");
        assert_eq!(find(&tokens, "name"), Some(TokenKind::Key));
    }

    #[test]
    fn equals_is_operator() {
        let tokens = tok("name = \"prytty\"");
        assert_eq!(find(&tokens, "="), Some(TokenKind::Operator));
    }

    #[test]
    fn string_value() {
        let tokens = tok("name = \"prytty\"");
        assert_eq!(find(&tokens, "\"prytty\""), Some(TokenKind::String));
    }

    #[test]
    fn single_quoted_string_value() {
        let tokens = tok("license = 'MIT'");
        assert_eq!(find(&tokens, "'MIT'"), Some(TokenKind::String));
    }

    #[test]
    fn boolean_true() {
        let tokens = tok("enabled = true");
        assert_eq!(find(&tokens, "true"), Some(TokenKind::Constant));
    }

    #[test]
    fn boolean_false() {
        let tokens = tok("debug = false");
        assert_eq!(find(&tokens, "false"), Some(TokenKind::Constant));
    }

    #[test]
    fn integer_value() {
        let tokens = tok("port = 8080");
        assert_eq!(find(&tokens, "8080"), Some(TokenKind::Number));
    }

    #[test]
    fn negative_integer() {
        let tokens = tok("offset = -5");
        assert_eq!(find(&tokens, "-5"), Some(TokenKind::Number));
    }

    #[test]
    fn float_value() {
        let tokens = tok("ratio = 0.5");
        assert_eq!(find(&tokens, "0.5"), Some(TokenKind::Number));
    }

    #[test]
    fn full_toml_snippet() {
        let input = "[package]\nname = \"foo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";
        let tokens = tok(input);
        assert_eq!(find(&tokens, "[package]"), Some(TokenKind::Attribute));
        assert_eq!(find(&tokens, "name"), Some(TokenKind::Key));
        assert_eq!(find(&tokens, "\"foo\""), Some(TokenKind::String));
        assert_eq!(find(&tokens, "version"), Some(TokenKind::Key));
    }
}

impl Grammar for TomlGrammar {
    fn tokenize<'a>(&self, input: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut pos = 0;
        let bytes = input.as_bytes();

        while pos < bytes.len() {
            // Newlines
            if bytes[pos] == b'\n' {
                tokens.push(Token { text: &input[pos..pos + 1], kind: TokenKind::Plain });
                pos += 1;
                continue;
            }

            // Skip leading whitespace
            if bytes[pos].is_ascii_whitespace() {
                let start = pos;
                while pos < bytes.len() && bytes[pos].is_ascii_whitespace() && bytes[pos] != b'\n' {
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Plain });
                continue;
            }

            // Comments
            if bytes[pos] == b'#' {
                let start = pos;
                while pos < bytes.len() && bytes[pos] != b'\n' {
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Comment });
                continue;
            }

            // Section headers: [section] or [[array]]
            if bytes[pos] == b'[' {
                let start = pos;
                while pos < bytes.len() && bytes[pos] != b'\n' {
                    if bytes[pos] == b']' {
                        pos += 1;
                        // Eat second ] for [[...]]
                        if pos < bytes.len() && bytes[pos] == b']' {
                            pos += 1;
                        }
                        break;
                    }
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Attribute });
                continue;
            }

            // Key = value lines
            // Find '=' on this line
            let line_start = pos;
            let mut eq_pos = None;
            let mut scan = pos;
            while scan < bytes.len() && bytes[scan] != b'\n' {
                if bytes[scan] == b'=' && eq_pos.is_none() {
                    eq_pos = Some(scan);
                    break;
                }
                scan += 1;
            }

            if let Some(eq) = eq_pos {
                // Key
                let key = input[line_start..eq].trim_end();
                if !key.is_empty() {
                    tokens.push(Token { text: &input[line_start..eq].trim_end(), kind: TokenKind::Key });
                    // Whitespace between key and =
                    let trimmed_len = key.len();
                    let raw_len = eq - line_start;
                    if raw_len > trimmed_len {
                        tokens.push(Token { text: &input[line_start + trimmed_len..eq], kind: TokenKind::Plain });
                    }
                }
                tokens.push(Token { text: &input[eq..eq + 1], kind: TokenKind::Operator });
                pos = eq + 1;

                // Value
                let val_start = pos;
                while pos < bytes.len() && bytes[pos] != b'\n' {
                    pos += 1;
                }
                let value = &input[val_start..pos];
                tokenize_toml_value(&mut tokens, value);
                continue;
            }

            // Fallback: rest of line
            while pos < bytes.len() && bytes[pos] != b'\n' {
                pos += 1;
            }
            tokens.push(Token { text: &input[line_start..pos], kind: TokenKind::Plain });
        }

        tokens
    }
}

fn tokenize_toml_value<'a>(tokens: &mut Vec<Token<'a>>, value: &'a str) {
    let leading = value.len() - value.trim_start().len();
    let trimmed = value.trim();

    if leading > 0 {
        tokens.push(Token { text: &value[..leading], kind: TokenKind::Plain });
    }

    if trimmed.is_empty() {
        return;
    }

    // Check for inline comment
    // Simple approach: find # that's not inside a string
    match trimmed.as_bytes()[0] {
        b'"' | b'\'' => {
            tokens.push(Token { text: trimmed, kind: TokenKind::String });
        }
        b't' if trimmed == "true" => {
            tokens.push(Token { text: trimmed, kind: TokenKind::Constant });
        }
        b'f' if trimmed == "false" => {
            tokens.push(Token { text: trimmed, kind: TokenKind::Constant });
        }
        b'[' => {
            tokens.push(Token { text: trimmed, kind: TokenKind::Punctuation });
        }
        b'{' => {
            tokens.push(Token { text: trimmed, kind: TokenKind::Punctuation });
        }
        b if b.is_ascii_digit() || b == b'-' || b == b'+' => {
            tokens.push(Token { text: trimmed, kind: TokenKind::Number });
        }
        _ => {
            tokens.push(Token { text: trimmed, kind: TokenKind::String });
        }
    }

    // Preserve trailing whitespace
    let trailing_start = leading + trimmed.len();
    if trailing_start < value.len() {
        tokens.push(Token { text: &value[trailing_start..], kind: TokenKind::Plain });
    }
}
