use crate::Grammar;
use prytty_core::{Token, TokenKind};

pub struct DiffGrammar;

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(input: &str) -> Vec<(TokenKind, &str)> {
        DiffGrammar.tokenize(input).into_iter().map(|t| (t.kind, t.text)).collect()
    }

    fn first_kind(tokens: &[(TokenKind, &str)]) -> Option<TokenKind> {
        tokens.first().map(|(k, _)| *k)
    }

    #[test]
    fn added_line_is_string() {
        let tokens = tok("+added line\n");
        assert_eq!(first_kind(&tokens), Some(TokenKind::String));
    }

    #[test]
    fn removed_line_is_keyword() {
        let tokens = tok("-removed line\n");
        assert_eq!(first_kind(&tokens), Some(TokenKind::Keyword));
    }

    #[test]
    fn context_line_is_plain() {
        let tokens = tok(" context line\n");
        assert_eq!(first_kind(&tokens), Some(TokenKind::Plain));
    }

    #[test]
    fn diff_header_is_label() {
        let tokens = tok("diff --git a/foo.rs b/foo.rs\n");
        assert_eq!(first_kind(&tokens), Some(TokenKind::Label));
    }

    #[test]
    fn old_file_header_is_label() {
        let tokens = tok("--- a/foo.rs\n");
        assert_eq!(first_kind(&tokens), Some(TokenKind::Label));
    }

    #[test]
    fn new_file_header_is_label() {
        let tokens = tok("+++ b/foo.rs\n");
        assert_eq!(first_kind(&tokens), Some(TokenKind::Label));
    }

    #[test]
    fn hunk_marker_is_attribute() {
        let tokens = tok("@@ -1,5 +1,7 @@\n");
        assert_eq!(first_kind(&tokens), Some(TokenKind::Attribute));
    }

    #[test]
    fn index_line_is_comment() {
        let tokens = tok("index abc1234..def5678 100644\n");
        assert_eq!(first_kind(&tokens), Some(TokenKind::Comment));
    }

    #[test]
    fn full_diff() {
        let input = concat!(
            "diff --git a/foo.rs b/foo.rs\n",
            "index abc..def 100644\n",
            "--- a/foo.rs\n",
            "+++ b/foo.rs\n",
            "@@ -1,3 +1,4 @@\n",
            " unchanged\n",
            "-removed\n",
            "+added\n",
        );
        let tokens = tok(input);
        // Each line → one token; check kinds by scanning text
        for (kind, text) in &tokens {
            if text.trim_end().starts_with("diff ") {
                assert_eq!(*kind, TokenKind::Label, "diff line should be Label");
            } else if text.trim_end().starts_with('+') && !text.starts_with("+++") {
                assert_eq!(*kind, TokenKind::String, "added line should be String");
            } else if text.trim_end().starts_with('-') && !text.starts_with("---") {
                assert_eq!(*kind, TokenKind::Keyword, "removed line should be Keyword");
            } else if text.trim_end().starts_with("@@ ") {
                assert_eq!(*kind, TokenKind::Attribute, "hunk should be Attribute");
            }
        }
    }

    #[test]
    fn no_trailing_newline() {
        // Last line without newline should still be tokenized
        let tokens = tok("+added");
        assert_eq!(first_kind(&tokens), Some(TokenKind::String));
    }
}

impl Grammar for DiffGrammar {
    fn tokenize<'a>(&self, input: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();

        for line in input.split_inclusive('\n') {
            let kind = classify_diff_line(line);
            tokens.push(Token { text: line, kind });
        }

        // Handle last line without trailing newline
        if !input.is_empty() && !input.ends_with('\n') {
            // Already handled by split_inclusive
        }

        tokens
    }
}

fn classify_diff_line(line: &str) -> TokenKind {
    let trimmed = line.trim_end();
    if trimmed.starts_with("diff ") {
        TokenKind::Label
    } else if trimmed.starts_with("index ") {
        TokenKind::Comment
    } else if trimmed.starts_with("--- ") || trimmed.starts_with("+++ ") {
        TokenKind::Label
    } else if trimmed.starts_with("@@ ") {
        TokenKind::Attribute
    } else if trimmed.starts_with('+') {
        TokenKind::String // green (string color in dark+ theme)
    } else if trimmed.starts_with('-') {
        TokenKind::Keyword // red-ish (keyword color = purple, but we remap)
    } else {
        TokenKind::Plain
    }
}
