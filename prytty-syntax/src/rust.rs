use crate::generic::advance_char;
use crate::Grammar;
use prytty_core::{Token, TokenKind};

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(input: &str) -> Vec<(TokenKind, &str)> {
        RustGrammar.tokenize(input).into_iter().map(|t| (t.kind, t.text)).collect()
    }

    fn find(tokens: &[(TokenKind, &str)], text: &str) -> Option<TokenKind> {
        tokens.iter().find(|(_, t)| *t == text).map(|(k, _)| *k)
    }

    #[test]
    fn keywords() {
        let tokens = tok("fn let use mod pub impl struct enum");
        for kw in &["fn", "let", "use", "mod", "pub", "impl", "struct", "enum"] {
            assert_eq!(find(&tokens, kw), Some(TokenKind::Keyword), "'{kw}' should be Keyword");
        }
    }

    #[test]
    fn flow_keywords() {
        let tokens = tok("if else match return while for loop break continue");
        for kw in &["if", "else", "match", "return", "while", "for", "loop", "break", "continue"] {
            assert_eq!(find(&tokens, kw), Some(TokenKind::Keyword), "'{kw}' should be Keyword");
        }
    }

    #[test]
    fn boolean_keywords() {
        let tokens = tok("true false");
        assert_eq!(find(&tokens, "true"), Some(TokenKind::Keyword));
        assert_eq!(find(&tokens, "false"), Some(TokenKind::Keyword));
    }

    #[test]
    fn primitive_types() {
        let tokens = tok("i32 u8 f64 bool char str usize");
        for ty in &["i32", "u8", "f64", "bool", "char", "str", "usize"] {
            assert_eq!(find(&tokens, ty), Some(TokenKind::Type), "'{ty}' should be Type");
        }
    }

    #[test]
    fn stdlib_types() {
        let tokens = tok("String Vec Option Result Box HashMap");
        for ty in &["String", "Vec", "Option", "Result", "Box", "HashMap"] {
            assert_eq!(find(&tokens, ty), Some(TokenKind::Type), "'{ty}' should be Type");
        }
    }

    #[test]
    fn builtins_without_bang() {
        let tokens = tok("Some None Ok Err");
        for b in &["Some", "None", "Ok", "Err"] {
            assert_eq!(find(&tokens, b), Some(TokenKind::Builtin), "'{b}' should be Builtin");
        }
    }

    #[test]
    fn macro_println_bang() {
        let tokens = tok("println!(\"hi\")");
        assert_eq!(find(&tokens, "println!"), Some(TokenKind::Builtin));
    }

    #[test]
    fn macro_assert_eq_bang() {
        let tokens = tok("assert_eq!(a, b)");
        assert_eq!(find(&tokens, "assert_eq!"), Some(TokenKind::Builtin));
    }

    #[test]
    fn macro_vec_bang() {
        let tokens = tok("vec![1, 2, 3]");
        assert_eq!(find(&tokens, "vec!"), Some(TokenKind::Builtin));
    }

    #[test]
    fn string_literal() {
        let tokens = tok("\"hello world\"");
        assert_eq!(
            tokens.iter().find(|(k, _)| *k == TokenKind::String).map(|(_, t)| *t),
            Some("\"hello world\"")
        );
    }

    #[test]
    fn string_with_escape() {
        let tokens = tok("\"hello\\nworld\"");
        assert_eq!(
            tokens.iter().find(|(k, _)| *k == TokenKind::String).map(|(_, t)| *t),
            Some("\"hello\\nworld\"")
        );
    }

    #[test]
    fn raw_string_no_hashes() {
        let tokens = tok("r\"raw string\"");
        let s = tokens.iter().find(|(k, _)| *k == TokenKind::String);
        assert!(s.is_some(), "expected String token for raw string");
        assert_eq!(s.unwrap().1, "r\"raw string\"");
    }

    #[test]
    fn raw_string_with_hashes() {
        // r#"has "quotes" inside"#
        let input = "r#\"has \\\"quotes\\\" inside\"#";
        let tokens = tok(input);
        let s = tokens.iter().find(|(k, _)| *k == TokenKind::String);
        assert!(s.is_some(), "expected String token for raw string with hashes");
    }

    #[test]
    fn char_literal() {
        let tokens = tok("'a'");
        assert_eq!(find(&tokens, "'a'"), Some(TokenKind::String));
    }

    #[test]
    fn char_literal_escaped() {
        let tokens = tok("'\\n'");
        assert_eq!(find(&tokens, "'\\n'"), Some(TokenKind::String));
    }

    #[test]
    fn lifetime() {
        let tokens = tok("'a");
        assert_eq!(find(&tokens, "'a"), Some(TokenKind::Variable));
    }

    #[test]
    fn integer_decimal() {
        let tokens = tok("42");
        assert_eq!(find(&tokens, "42"), Some(TokenKind::Number));
    }

    #[test]
    fn integer_with_suffix() {
        let tokens = tok("42u32");
        assert_eq!(find(&tokens, "42u32"), Some(TokenKind::Number));
    }

    #[test]
    fn float() {
        let tokens = tok("3.14");
        assert_eq!(find(&tokens, "3.14"), Some(TokenKind::Number));
    }

    #[test]
    fn hex_number() {
        let tokens = tok("0xFF");
        assert_eq!(find(&tokens, "0xFF"), Some(TokenKind::Number));
    }

    #[test]
    fn binary_number() {
        let tokens = tok("0b1010");
        assert_eq!(find(&tokens, "0b1010"), Some(TokenKind::Number));
    }

    #[test]
    fn line_comment() {
        let tokens = tok("// this is a comment\n");
        assert_eq!(find(&tokens, "// this is a comment"), Some(TokenKind::Comment));
    }

    #[test]
    fn block_comment() {
        let tokens = tok("/* block */");
        assert_eq!(find(&tokens, "/* block */"), Some(TokenKind::Comment));
    }

    #[test]
    fn attribute_outer() {
        let tokens = tok("#[derive(Debug)]");
        assert_eq!(find(&tokens, "#[derive(Debug)]"), Some(TokenKind::Attribute));
    }

    #[test]
    fn attribute_inner() {
        let tokens = tok("#![allow(unused)]");
        assert_eq!(find(&tokens, "#![allow(unused)]"), Some(TokenKind::Attribute));
    }

    #[test]
    fn function_call() {
        let tokens = tok("foo(x)");
        assert_eq!(find(&tokens, "foo"), Some(TokenKind::Function));
    }

    #[test]
    fn operator_double_eq() {
        let tokens = tok("a == b");
        assert_eq!(find(&tokens, "=="), Some(TokenKind::Operator));
    }

    #[test]
    fn operator_fat_arrow() {
        let tokens = tok("x => y");
        assert_eq!(find(&tokens, "=>"), Some(TokenKind::Operator));
    }

    #[test]
    fn punctuation_brace() {
        let tokens = tok("{");
        assert_eq!(find(&tokens, "{"), Some(TokenKind::Punctuation));
    }

    #[test]
    fn pascal_case_is_type() {
        let tokens = tok("MyStruct");
        assert_eq!(find(&tokens, "MyStruct"), Some(TokenKind::Type));
    }

    #[test]
    fn full_fn_declaration() {
        let src = "pub fn add(a: i32, b: i32) -> i32 { a + b }";
        let tokens = tok(src);
        assert_eq!(find(&tokens, "pub"), Some(TokenKind::Keyword));
        assert_eq!(find(&tokens, "fn"), Some(TokenKind::Keyword));
        assert_eq!(find(&tokens, "add"), Some(TokenKind::Function));
        assert_eq!(find(&tokens, "i32"), Some(TokenKind::Type));
    }
}

pub struct RustGrammar;

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
    "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
    "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true",
    "type", "unsafe", "use", "where", "while", "yield",
];

const TYPES: &[&str] = &[
    "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "str", "u8",
    "u16", "u32", "u64", "u128", "usize", "String", "Vec", "Option", "Result", "Box", "Rc",
    "Arc", "HashMap", "HashSet", "BTreeMap", "BTreeSet", "PhantomData",
];

const BUILTINS: &[&str] = &[
    "println", "eprintln", "print", "eprint", "format", "write", "writeln", "panic", "todo",
    "unimplemented", "unreachable", "assert", "assert_eq", "assert_ne", "dbg", "cfg", "vec",
    "Some", "None", "Ok", "Err",
];

impl Grammar for RustGrammar {
    fn tokenize<'a>(&self, input: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut pos = 0;
        let bytes = input.as_bytes();

        while pos < bytes.len() {
            // Line comments
            if pos + 1 < bytes.len() && bytes[pos] == b'/' && bytes[pos + 1] == b'/' {
                let start = pos;
                while pos < bytes.len() && bytes[pos] != b'\n' {
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Comment });
                continue;
            }

            // Block comments
            if pos + 1 < bytes.len() && bytes[pos] == b'/' && bytes[pos + 1] == b'*' {
                let start = pos;
                pos += 2;
                while pos + 1 < bytes.len() && !(bytes[pos] == b'*' && bytes[pos + 1] == b'/') {
                    pos += 1;
                }
                if pos + 1 < bytes.len() {
                    pos += 2;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Comment });
                continue;
            }

            // Attributes: #[...] or #![...]
            if bytes[pos] == b'#' && pos + 1 < bytes.len() && (bytes[pos + 1] == b'[' || (bytes[pos + 1] == b'!' && pos + 2 < bytes.len() && bytes[pos + 2] == b'[')) {
                let start = pos;
                let mut depth: i32 = 0;
                while pos < bytes.len() {
                    if bytes[pos] == b'[' {
                        depth += 1;
                    } else if bytes[pos] == b']' {
                        depth -= 1;
                        if depth <= 0 {
                            pos += 1;
                            break;
                        }
                    }
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Attribute });
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
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::String });
                continue;
            }

            // Raw strings: r"..." or r#"..."#
            if bytes[pos] == b'r' && pos + 1 < bytes.len() && (bytes[pos + 1] == b'"' || bytes[pos + 1] == b'#') {
                let start = pos;
                pos += 1;
                let mut hashes = 0u32;
                while pos < bytes.len() && bytes[pos] == b'#' {
                    hashes += 1;
                    pos += 1;
                }
                if pos < bytes.len() && bytes[pos] == b'"' {
                    pos += 1;
                    'raw: loop {
                        if pos >= bytes.len() {
                            break;
                        }
                        if bytes[pos] == b'"' {
                            pos += 1;
                            let mut end_hashes = 0u32;
                            while pos < bytes.len() && bytes[pos] == b'#' && end_hashes < hashes {
                                end_hashes += 1;
                                pos += 1;
                            }
                            if end_hashes == hashes {
                                break 'raw;
                            }
                        } else {
                            pos += 1;
                        }
                    }
                    tokens.push(Token { text: &input[start..pos], kind: TokenKind::String });
                    continue;
                }
                // Not actually a raw string, backtrack
                pos = start;
            }

            // Char literals
            if bytes[pos] == b'\'' && pos + 2 < bytes.len() {
                // Peek ahead: must be 'x' or '\x'
                let start = pos;
                pos += 1;
                if pos < bytes.len() && bytes[pos] == b'\\' {
                    pos += 1;
                }
                if pos < bytes.len() {
                    pos += 1;
                }
                if pos < bytes.len() && bytes[pos] == b'\'' {
                    pos += 1;
                    tokens.push(Token { text: &input[start..pos], kind: TokenKind::String });
                    continue;
                }
                // Not a char literal, might be a lifetime
                pos = start;
            }

            // Lifetime: 'a
            if bytes[pos] == b'\'' && pos + 1 < bytes.len() && bytes[pos + 1].is_ascii_alphabetic() {
                let start = pos;
                pos += 1;
                while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Variable });
                continue;
            }

            // Numbers
            if bytes[pos].is_ascii_digit() {
                let start = pos;
                // Hex, oct, bin prefix
                if bytes[pos] == b'0' && pos + 1 < bytes.len() {
                    match bytes[pos + 1] {
                        b'x' | b'X' | b'o' | b'O' | b'b' | b'B' => {
                            pos += 2;
                            while pos < bytes.len() && (bytes[pos].is_ascii_hexdigit() || bytes[pos] == b'_') {
                                pos += 1;
                            }
                            tokens.push(Token { text: &input[start..pos], kind: TokenKind::Number });
                            continue;
                        }
                        _ => {}
                    }
                }
                while pos < bytes.len() && (bytes[pos].is_ascii_digit() || bytes[pos] == b'_' || bytes[pos] == b'.') {
                    // Don't eat .. (range operator)
                    if bytes[pos] == b'.' && pos + 1 < bytes.len() && bytes[pos + 1] == b'.' {
                        break;
                    }
                    pos += 1;
                }
                // Type suffix: u8, i32, f64, etc.
                if pos < bytes.len() && (bytes[pos] == b'u' || bytes[pos] == b'i' || bytes[pos] == b'f') {
                    while pos < bytes.len() && bytes[pos].is_ascii_alphanumeric() {
                        pos += 1;
                    }
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Number });
                continue;
            }

            // Operators (multi-char)
            if b"=!<>+-*/%&|^~?.".contains(&bytes[pos]) {
                let start = pos;
                pos += 1;
                // Absorb second char of two-char operators
                if pos < bytes.len() && b"=>&|.".contains(&bytes[pos]) {
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Operator });
                continue;
            }

            // Punctuation
            if b"(){}[],:;@#".contains(&bytes[pos]) {
                let start = pos;
                pos += 1;
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Punctuation });
                continue;
            }

            // Identifiers and keywords
            if bytes[pos].is_ascii_alphabetic() || bytes[pos] == b'_' {
                let start = pos;
                while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                    pos += 1;
                }
                let word = &input[start..pos];

                // Check if followed by ! (macro)
                if pos < bytes.len() && bytes[pos] == b'!' {
                    if BUILTINS.contains(&word) {
                        pos += 1; // include the !
                        tokens.push(Token { text: &input[start..pos], kind: TokenKind::Builtin });
                        continue;
                    }
                }

                let kind = if KEYWORDS.contains(&word) {
                    TokenKind::Keyword
                } else if TYPES.contains(&word) {
                    TokenKind::Type
                } else if BUILTINS.contains(&word) {
                    TokenKind::Builtin
                } else if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                    TokenKind::Type // PascalCase = likely a type
                } else if pos < bytes.len() && bytes[pos] == b'(' {
                    TokenKind::Function // followed by ( = likely a function call
                } else {
                    TokenKind::Plain
                };

                tokens.push(Token { text: word, kind });
                continue;
            }

            // Whitespace and anything else (including multi-byte UTF-8)
            let start = pos;
            pos = advance_char(bytes, pos);
            tokens.push(Token { text: &input[start..pos], kind: TokenKind::Plain });
        }

        tokens
    }
}
