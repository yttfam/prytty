use crate::Grammar;
use prytty_core::{Token, TokenKind};

pub struct YamlGrammar;

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(input: &str) -> Vec<(TokenKind, &str)> {
        YamlGrammar.tokenize(input).into_iter().map(|t| (t.kind, t.text)).collect()
    }

    fn find(tokens: &[(TokenKind, &str)], text: &str) -> Option<TokenKind> {
        tokens.iter().find(|(_, t)| *t == text).map(|(k, _)| *k)
    }

    fn has_kind(tokens: &[(TokenKind, &str)], kind: TokenKind) -> bool {
        tokens.iter().any(|(k, _)| *k == kind)
    }

    #[test]
    fn document_start_marker() {
        let tokens = tok("---");
        assert_eq!(find(&tokens, "---"), Some(TokenKind::Punctuation));
    }

    #[test]
    fn document_end_marker() {
        let tokens = tok("...");
        assert_eq!(find(&tokens, "..."), Some(TokenKind::Punctuation));
    }

    #[test]
    fn comment_line() {
        let tokens = tok("# a comment");
        assert_eq!(find(&tokens, "# a comment"), Some(TokenKind::Comment));
    }

    #[test]
    fn indented_comment() {
        let tokens = tok("  # indented comment");
        assert_eq!(find(&tokens, "# indented comment"), Some(TokenKind::Comment));
    }

    #[test]
    fn simple_key_value() {
        let tokens = tok("name: alice");
        assert_eq!(find(&tokens, "name"), Some(TokenKind::Key));
        assert_eq!(find(&tokens, ":"), Some(TokenKind::Punctuation));
        assert_eq!(find(&tokens, "alice"), Some(TokenKind::String));
    }

    #[test]
    fn key_with_number_value() {
        let tokens = tok("port: 8080");
        assert_eq!(find(&tokens, "port"), Some(TokenKind::Key));
        assert_eq!(find(&tokens, "8080"), Some(TokenKind::Number));
    }

    #[test]
    fn boolean_true_value() {
        let tokens = tok("enabled: true");
        assert_eq!(find(&tokens, "true"), Some(TokenKind::Constant));
    }

    #[test]
    fn boolean_false_value() {
        let tokens = tok("debug: false");
        assert_eq!(find(&tokens, "false"), Some(TokenKind::Constant));
    }

    #[test]
    fn boolean_yes_no() {
        let tokens_yes = tok("active: yes");
        let tokens_no = tok("active: no");
        assert_eq!(find(&tokens_yes, "yes"), Some(TokenKind::Constant));
        assert_eq!(find(&tokens_no, "no"), Some(TokenKind::Constant));
    }

    #[test]
    fn null_value() {
        let tokens = tok("x: null");
        assert_eq!(find(&tokens, "null"), Some(TokenKind::Constant));
    }

    #[test]
    fn tilde_null() {
        let tokens = tok("x: ~");
        assert_eq!(find(&tokens, "~"), Some(TokenKind::Constant));
    }

    #[test]
    fn quoted_string_value() {
        let tokens = tok("msg: \"hello world\"");
        assert_eq!(find(&tokens, "\"hello world\""), Some(TokenKind::String));
    }

    #[test]
    fn list_item_with_key() {
        // "- key: value" line
        let tokens = tok("- name: bob");
        assert!(has_kind(&tokens, TokenKind::Punctuation), "expected Punctuation for list dash");
        assert_eq!(find(&tokens, "name"), Some(TokenKind::Key));
        assert_eq!(find(&tokens, "bob"), Some(TokenKind::String));
    }

    #[test]
    fn list_item_bare_value() {
        let tokens = tok("- hello");
        assert!(has_kind(&tokens, TokenKind::Punctuation), "expected Punctuation for '- '");
        assert_eq!(find(&tokens, "hello"), Some(TokenKind::String));
    }

    #[test]
    fn nested_keys() {
        let input = "server:\n  host: localhost\n  port: 80";
        let tokens = tok(input);
        assert_eq!(find(&tokens, "server"), Some(TokenKind::Key));
        assert_eq!(find(&tokens, "host"), Some(TokenKind::Key));
        assert_eq!(find(&tokens, "port"), Some(TokenKind::Key));
        assert_eq!(find(&tokens, "80"), Some(TokenKind::Number));
    }
}

impl Grammar for YamlGrammar {
    fn tokenize<'a>(&self, input: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();

        for line in input.split_inclusive('\n') {
            // Strip trailing \n for line analysis, emit it separately
            let (content, newline) = if line.ends_with('\n') {
                (&line[..line.len() - 1], Some(&line[line.len() - 1..]))
            } else {
                (line, None)
            };
            if !content.is_empty() {
                tokenize_yaml_line(&mut tokens, content);
            }
            if let Some(nl) = newline {
                tokens.push(Token { text: nl, kind: TokenKind::Plain });
            }
        }

        tokens
    }
}

fn tokenize_yaml_line<'a>(tokens: &mut Vec<Token<'a>>, line: &'a str) {
    let trimmed = line.trim_start();

    // Document markers
    if trimmed == "---" || trimmed == "..." {
        tokens.push(Token { text: line, kind: TokenKind::Punctuation });
        return;
    }

    // Comments
    if trimmed.starts_with('#') {
        // Leading whitespace
        if line.len() > trimmed.len() {
            tokens.push(Token { text: &line[..line.len() - trimmed.len()], kind: TokenKind::Plain });
        }
        tokens.push(Token { text: trimmed, kind: TokenKind::Comment });
        return;
    }

    // Key: value lines
    if let Some(colon_pos) = find_yaml_colon(trimmed) {
        let indent_len = line.len() - trimmed.len();
        if indent_len > 0 {
            tokens.push(Token { text: &line[..indent_len], kind: TokenKind::Plain });
        }

        // Handle list prefix "- "
        let key_start;
        if trimmed.starts_with("- ") {
            tokens.push(Token { text: &trimmed[..2], kind: TokenKind::Punctuation });
            key_start = 2;
        } else {
            key_start = 0;
        }

        // colon_pos is relative to the text after any "- " prefix
        let abs_colon = key_start + colon_pos;
        let key = &trimmed[key_start..abs_colon];
        tokens.push(Token { text: key, kind: TokenKind::Key });
        tokens.push(Token { text: &trimmed[abs_colon..abs_colon + 1], kind: TokenKind::Punctuation });

        let rest = &trimmed[abs_colon + 1..];
        if !rest.is_empty() {
            tokenize_yaml_value(tokens, rest);
        }
        return;
    }

    // List items without key
    if trimmed.starts_with("- ") {
        let indent_len = line.len() - trimmed.len();
        if indent_len > 0 {
            tokens.push(Token { text: &line[..indent_len], kind: TokenKind::Plain });
        }
        tokens.push(Token { text: &trimmed[..2], kind: TokenKind::Punctuation });
        tokenize_yaml_value(tokens, &trimmed[2..]);
        return;
    }

    // Fallback
    tokens.push(Token { text: line, kind: TokenKind::Plain });
}

fn tokenize_yaml_value<'a>(tokens: &mut Vec<Token<'a>>, value: &'a str) {
    let trimmed = value.trim_start();
    if value.len() > trimmed.len() {
        tokens.push(Token { text: &value[..value.len() - trimmed.len()], kind: TokenKind::Plain });
    }

    if trimmed.is_empty() {
        return;
    }

    match trimmed {
        "true" | "false" | "yes" | "no" | "on" | "off" | "null" | "~" => {
            tokens.push(Token { text: trimmed, kind: TokenKind::Constant });
        }
        _ if trimmed.starts_with('"') || trimmed.starts_with('\'') => {
            tokens.push(Token { text: trimmed, kind: TokenKind::String });
        }
        _ if trimmed.bytes().next().is_some_and(|b| b.is_ascii_digit() || b == b'-') => {
            // Try to see if it's a number
            if trimmed.parse::<f64>().is_ok() || trimmed.parse::<i64>().is_ok() {
                tokens.push(Token { text: trimmed, kind: TokenKind::Number });
            } else {
                tokens.push(Token { text: trimmed, kind: TokenKind::String });
            }
        }
        _ if trimmed.starts_with('#') => {
            tokens.push(Token { text: trimmed, kind: TokenKind::Comment });
        }
        _ => {
            // Check for inline comment
            if let Some(comment_pos) = trimmed.find(" #") {
                tokens.push(Token { text: &trimmed[..comment_pos], kind: TokenKind::String });
                tokens.push(Token { text: &trimmed[comment_pos..comment_pos + 1], kind: TokenKind::Plain });
                tokens.push(Token { text: &trimmed[comment_pos + 1..], kind: TokenKind::Comment });
            } else {
                tokens.push(Token { text: trimmed, kind: TokenKind::String });
            }
        }
    }
}

fn find_yaml_colon(line: &str) -> Option<usize> {
    let trimmed = if line.starts_with("- ") { &line[2..] } else { line };
    // Find first unquoted colon followed by space or end
    let mut in_quote = false;
    let mut quote_char = b'"';
    for (i, &b) in trimmed.as_bytes().iter().enumerate() {
        if in_quote {
            if b == quote_char {
                in_quote = false;
            }
            continue;
        }
        if b == b'"' || b == b'\'' {
            in_quote = true;
            quote_char = b;
            continue;
        }
        if b == b':' && (i + 1 >= trimmed.len() || trimmed.as_bytes()[i + 1] == b' ') {
            return Some(i);
        }
    }
    None
}

