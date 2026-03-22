/// A syntax token: a span of text with a semantic kind.
#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub text: &'a str,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Keyword,
    Type,
    Function,
    String,
    Number,
    Comment,
    Operator,
    Punctuation,
    Variable,
    Constant,
    Attribute,
    Builtin,
    Label,      // log levels, diff markers
    Key,        // JSON/YAML/TOML keys
    Escape,     // escape sequences in strings
    Url,
    Path,
    Ip,
    Timestamp,
    Plain,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_kind_eq() {
        assert_eq!(TokenKind::Keyword, TokenKind::Keyword);
        assert_ne!(TokenKind::Keyword, TokenKind::Type);
    }

    #[test]
    fn token_kind_copy() {
        let k = TokenKind::String;
        let k2 = k;
        assert_eq!(k, k2);
    }

    #[test]
    fn token_kind_debug() {
        let s = format!("{:?}", TokenKind::Comment);
        assert_eq!(s, "Comment");
    }

    #[test]
    fn all_variants_are_distinct() {
        use std::collections::HashSet;
        let kinds = [
            TokenKind::Keyword, TokenKind::Type, TokenKind::Function,
            TokenKind::String, TokenKind::Number, TokenKind::Comment,
            TokenKind::Operator, TokenKind::Punctuation, TokenKind::Variable,
            TokenKind::Constant, TokenKind::Attribute, TokenKind::Builtin,
            TokenKind::Label, TokenKind::Key, TokenKind::Escape,
            TokenKind::Url, TokenKind::Path, TokenKind::Ip,
            TokenKind::Timestamp, TokenKind::Plain,
        ];
        let set: HashSet<_> = kinds.iter().copied().collect();
        assert_eq!(set.len(), kinds.len());
    }

    #[test]
    fn token_stores_text_and_kind() {
        let src = "hello";
        let tok = Token { text: src, kind: TokenKind::Plain };
        assert_eq!(tok.text, "hello");
        assert_eq!(tok.kind, TokenKind::Plain);
    }

    #[test]
    fn token_clone() {
        let src = "world";
        let tok = Token { text: src, kind: TokenKind::Keyword };
        let tok2 = tok.clone();
        assert_eq!(tok2.text, tok.text);
        assert_eq!(tok2.kind, tok.kind);
    }
}
