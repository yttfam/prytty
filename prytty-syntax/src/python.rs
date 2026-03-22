use crate::generic::advance_char;
use crate::Grammar;
use prytty_core::{Token, TokenKind};

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(input: &str) -> Vec<(TokenKind, &str)> {
        PythonGrammar.tokenize(input).into_iter().map(|t| (t.kind, t.text)).collect()
    }

    fn find(tokens: &[(TokenKind, &str)], text: &str) -> Option<TokenKind> {
        tokens.iter().find(|(_, t)| *t == text).map(|(k, _)| *k)
    }

    #[test]
    fn keywords() {
        let tokens = tok("def class if else for while return import from");
        for kw in &["def", "class", "if", "else", "for", "while", "return", "import", "from"] {
            assert_eq!(find(&tokens, kw), Some(TokenKind::Keyword), "'{kw}' should be Keyword");
        }
    }

    #[test]
    fn more_keywords() {
        let tokens = tok("and or not in is lambda yield async await pass break continue");
        for kw in &["and", "or", "not", "in", "is", "lambda", "yield", "async", "await", "pass", "break", "continue"] {
            assert_eq!(find(&tokens, kw), Some(TokenKind::Keyword), "'{kw}' should be Keyword");
        }
    }

    #[test]
    fn none_true_false_are_keywords() {
        let tokens = tok("None True False");
        assert_eq!(find(&tokens, "None"), Some(TokenKind::Keyword));
        assert_eq!(find(&tokens, "True"), Some(TokenKind::Keyword));
        assert_eq!(find(&tokens, "False"), Some(TokenKind::Keyword));
    }

    #[test]
    fn builtins() {
        let tokens = tok("print(len(range(10)))");
        assert_eq!(find(&tokens, "print"), Some(TokenKind::Builtin));
        assert_eq!(find(&tokens, "len"), Some(TokenKind::Builtin));
        assert_eq!(find(&tokens, "range"), Some(TokenKind::Builtin));
    }

    #[test]
    fn decorator() {
        let tokens = tok("@property");
        assert_eq!(find(&tokens, "@property"), Some(TokenKind::Attribute));
    }

    #[test]
    fn decorator_with_dot() {
        let tokens = tok("@staticmethod");
        assert_eq!(find(&tokens, "@staticmethod"), Some(TokenKind::Attribute));
    }

    #[test]
    fn double_quoted_string() {
        let tokens = tok("\"hello\"");
        assert_eq!(
            tokens.iter().find(|(k, _)| *k == TokenKind::String).map(|(_, t)| *t),
            Some("\"hello\"")
        );
    }

    #[test]
    fn single_quoted_string() {
        let tokens = tok("'world'");
        assert_eq!(
            tokens.iter().find(|(k, _)| *k == TokenKind::String).map(|(_, t)| *t),
            Some("'world'")
        );
    }

    #[test]
    fn triple_double_quoted_string() {
        let tokens = tok("\"\"\"multi\nline\"\"\"");
        assert_eq!(
            tokens.iter().find(|(k, _)| *k == TokenKind::String).map(|(_, t)| *t),
            Some("\"\"\"multi\nline\"\"\"")
        );
    }

    #[test]
    fn triple_single_quoted_string() {
        let tokens = tok("'''also multi'''");
        assert_eq!(
            tokens.iter().find(|(k, _)| *k == TokenKind::String).map(|(_, t)| *t),
            Some("'''also multi'''")
        );
    }

    #[test]
    fn f_string() {
        let tokens = tok("f\"value={x}\"");
        assert_eq!(
            tokens.iter().find(|(k, _)| *k == TokenKind::String).map(|(_, t)| *t),
            Some("f\"value={x}\"")
        );
    }

    #[test]
    fn b_string() {
        let tokens = tok("b\"bytes\"");
        assert_eq!(
            tokens.iter().find(|(k, _)| *k == TokenKind::String).map(|(_, t)| *t),
            Some("b\"bytes\"")
        );
    }

    #[test]
    fn integer() {
        let tokens = tok("42");
        assert_eq!(find(&tokens, "42"), Some(TokenKind::Number));
    }

    #[test]
    fn hex_number() {
        let tokens = tok("0xFF");
        assert_eq!(find(&tokens, "0xFF"), Some(TokenKind::Number));
    }

    #[test]
    fn float() {
        let tokens = tok("3.14");
        assert_eq!(find(&tokens, "3.14"), Some(TokenKind::Number));
    }

    #[test]
    fn complex_number() {
        let tokens = tok("2j");
        assert_eq!(find(&tokens, "2j"), Some(TokenKind::Number));
    }

    #[test]
    fn comment() {
        let tokens = tok("# this is a comment");
        assert_eq!(find(&tokens, "# this is a comment"), Some(TokenKind::Comment));
    }

    #[test]
    fn function_call_is_function() {
        let tokens = tok("foo(x)");
        assert_eq!(find(&tokens, "foo"), Some(TokenKind::Function));
    }

    #[test]
    fn class_name_is_type() {
        // After the `class` keyword, the class name itself gets tokenized as Type (uppercase)
        let tokens = tok("MyClass");
        assert_eq!(find(&tokens, "MyClass"), Some(TokenKind::Type));
    }

    #[test]
    fn dunder_is_attribute() {
        let tokens = tok("__init__");
        assert_eq!(find(&tokens, "__init__"), Some(TokenKind::Attribute));
    }

    #[test]
    fn full_function_def() {
        let src = "def greet(name):\n    print(name)";
        let tokens = tok(src);
        assert_eq!(find(&tokens, "def"), Some(TokenKind::Keyword));
        assert_eq!(find(&tokens, "greet"), Some(TokenKind::Function));
        assert_eq!(find(&tokens, "print"), Some(TokenKind::Builtin));
    }
}

pub struct PythonGrammar;

const KEYWORDS: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
    "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
    "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return",
    "try", "while", "with", "yield",
];

const BUILTINS: &[&str] = &[
    "print", "len", "range", "int", "str", "float", "list", "dict", "set", "tuple", "bool",
    "type", "isinstance", "issubclass", "hasattr", "getattr", "setattr", "super", "property",
    "staticmethod", "classmethod", "enumerate", "zip", "map", "filter", "sorted", "reversed",
    "open", "input", "repr", "format", "vars", "dir", "help", "id", "hash", "callable",
    "iter", "next", "abs", "round", "min", "max", "sum", "any", "all", "ord", "chr", "hex",
    "oct", "bin",
];

impl Grammar for PythonGrammar {
    fn tokenize<'a>(&self, input: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut pos = 0;
        let bytes = input.as_bytes();

        while pos < bytes.len() {
            // Comments
            if bytes[pos] == b'#' {
                let start = pos;
                while pos < bytes.len() && bytes[pos] != b'\n' {
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Comment });
                continue;
            }

            // Triple-quoted strings
            if pos + 2 < bytes.len()
                && ((bytes[pos] == b'"' && bytes[pos + 1] == b'"' && bytes[pos + 2] == b'"')
                    || (bytes[pos] == b'\'' && bytes[pos + 1] == b'\'' && bytes[pos + 2] == b'\''))
            {
                let quote = bytes[pos];
                let start = pos;
                pos += 3;
                loop {
                    if pos + 2 >= bytes.len() {
                        pos = bytes.len();
                        break;
                    }
                    if bytes[pos] == quote && bytes[pos + 1] == quote && bytes[pos + 2] == quote {
                        pos += 3;
                        break;
                    }
                    if bytes[pos] == b'\\' && pos + 1 < bytes.len() {
                        pos += 1;
                    }
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::String });
                continue;
            }

            // f-strings, b-strings, r-strings prefix
            if (bytes[pos] == b'f' || bytes[pos] == b'b' || bytes[pos] == b'r')
                && pos + 1 < bytes.len()
                && (bytes[pos + 1] == b'"' || bytes[pos + 1] == b'\'')
            {
                let start = pos;
                pos += 1;
                let quote = bytes[pos];
                pos += 1;
                while pos < bytes.len() && bytes[pos] != quote {
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

            // Regular strings
            if bytes[pos] == b'"' || bytes[pos] == b'\'' {
                let quote = bytes[pos];
                let start = pos;
                pos += 1;
                while pos < bytes.len() && bytes[pos] != quote && bytes[pos] != b'\n' {
                    if bytes[pos] == b'\\' && pos + 1 < bytes.len() {
                        pos += 1;
                    }
                    pos += 1;
                }
                if pos < bytes.len() && bytes[pos] == quote {
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::String });
                continue;
            }

            // Decorators
            if bytes[pos] == b'@' {
                let start = pos;
                pos += 1;
                while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_' || bytes[pos] == b'.') {
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Attribute });
                continue;
            }

            // Numbers
            if bytes[pos].is_ascii_digit() {
                let start = pos;
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
                    pos += 1;
                }
                // Scientific notation
                if pos < bytes.len() && (bytes[pos] == b'e' || bytes[pos] == b'E') {
                    pos += 1;
                    if pos < bytes.len() && (bytes[pos] == b'+' || bytes[pos] == b'-') {
                        pos += 1;
                    }
                    while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                        pos += 1;
                    }
                }
                // Complex suffix
                if pos < bytes.len() && bytes[pos] == b'j' {
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Number });
                continue;
            }

            // Operators
            if b"=!<>+-*/%&|^~".contains(&bytes[pos]) {
                let start = pos;
                pos += 1;
                if pos < bytes.len() && b"=*>".contains(&bytes[pos]) {
                    pos += 1;
                }
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Operator });
                continue;
            }

            // Punctuation
            if b"(){}[],:;.\\".contains(&bytes[pos]) {
                let start = pos;
                pos += 1;
                tokens.push(Token { text: &input[start..pos], kind: TokenKind::Punctuation });
                continue;
            }

            // Identifiers
            if bytes[pos].is_ascii_alphabetic() || bytes[pos] == b'_' {
                let start = pos;
                while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                    pos += 1;
                }
                let word = &input[start..pos];
                let kind = if KEYWORDS.contains(&word) {
                    TokenKind::Keyword
                } else if BUILTINS.contains(&word) {
                    TokenKind::Builtin
                } else if word.starts_with("__") && word.ends_with("__") {
                    TokenKind::Attribute // dunder methods
                } else if pos < bytes.len() && bytes[pos] == b'(' {
                    TokenKind::Function
                } else if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                    TokenKind::Type
                } else {
                    TokenKind::Plain
                };
                tokens.push(Token { text: word, kind });
                continue;
            }

            // Everything else (including multi-byte UTF-8)
            let start = pos;
            pos = advance_char(bytes, pos);
            tokens.push(Token { text: &input[start..pos], kind: TokenKind::Plain });
        }

        tokens
    }
}
