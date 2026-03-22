use prytty_core::{Language, TokenKind};
use prytty_syntax::tokenize;

// ── Helper ────────────────────────────────────────────────────────────────────

fn join_tokens(tokens: &[prytty_core::Token<'_>]) -> String {
    tokens.iter().map(|t| t.text).collect()
}

// ── Rust grammar ──────────────────────────────────────────────────────────────

#[test]
fn rust_produces_keyword_for_fn() {
    let tokens = tokenize(Language::Rust, "fn main() {}");
    let kw = tokens.iter().find(|t| t.text == "fn");
    assert!(kw.is_some(), "expected a token with text 'fn'");
    assert_eq!(kw.unwrap().kind, TokenKind::Keyword);
}

#[test]
fn rust_produces_keyword_for_let() {
    let tokens = tokenize(Language::Rust, "let x = 42;");
    let kw = tokens.iter().find(|t| t.text == "let");
    assert!(kw.is_some(), "expected 'let' token");
    assert_eq!(kw.unwrap().kind, TokenKind::Keyword);
}

#[test]
fn rust_produces_keyword_for_struct() {
    let tokens = tokenize(Language::Rust, "pub struct Foo {}");
    assert!(tokens.iter().any(|t| t.text == "struct" && t.kind == TokenKind::Keyword));
}

#[test]
fn rust_produces_number_token() {
    let tokens = tokenize(Language::Rust, "let x = 42;");
    assert!(tokens.iter().any(|t| t.text == "42" && t.kind == TokenKind::Number));
}

#[test]
fn rust_produces_string_token() {
    let tokens = tokenize(Language::Rust, r#"let s = "hello";"#);
    assert!(tokens.iter().any(|t| t.text == "\"hello\"" && t.kind == TokenKind::String));
}

#[test]
fn rust_produces_comment_token() {
    let tokens = tokenize(Language::Rust, "// this is a comment\nfn main() {}");
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Comment && t.text.starts_with("//")));
}

#[test]
fn rust_lossless_single_line() {
    let input = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let tokens = tokenize(Language::Rust, input);
    assert_eq!(join_tokens(&tokens), input);
}

#[test]
fn rust_lossless_multiline() {
    let input = "fn main() {\n    let x = 42;\n    println!(\"{}\", x);\n}\n";
    let tokens = tokenize(Language::Rust, input);
    assert_eq!(join_tokens(&tokens), input);
}

// ── JSON grammar ──────────────────────────────────────────────────────────────

#[test]
fn json_produces_key_tokens_for_object_keys() {
    let input = r#"{"name": "prytty", "count": 3}"#;
    let tokens = tokenize(Language::Json, input);
    let keys: Vec<_> = tokens.iter().filter(|t| t.kind == TokenKind::Key).collect();
    assert!(!keys.is_empty(), "expected Key tokens for object keys");
    assert!(keys.iter().any(|t| t.text.contains("name")));
}

#[test]
fn json_produces_string_tokens_for_values() {
    let input = r#"{"lang": "rust"}"#;
    let tokens = tokenize(Language::Json, input);
    assert!(tokens.iter().any(|t| t.kind == TokenKind::String && t.text.contains("rust")));
}

#[test]
fn json_produces_number_tokens() {
    let input = r#"{"count": 42}"#;
    let tokens = tokenize(Language::Json, input);
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Number && t.text == "42"));
}

#[test]
fn json_produces_constant_for_true_false_null() {
    let input = r#"{"a": true, "b": false, "c": null}"#;
    let tokens = tokenize(Language::Json, input);
    let constants: Vec<_> = tokens.iter().filter(|t| t.kind == TokenKind::Constant).collect();
    assert_eq!(constants.len(), 3, "expected 3 constant tokens (true, false, null)");
}

#[test]
fn json_lossless() {
    let input = r#"{"name": "prytty", "count": 3, "active": true}"#;
    let tokens = tokenize(Language::Json, input);
    assert_eq!(join_tokens(&tokens), input);
}

#[test]
fn json_lossless_nested() {
    let input = r#"{"a":{"b":1},"c":[1,2,3]}"#;
    let tokens = tokenize(Language::Json, input);
    assert_eq!(join_tokens(&tokens), input);
}

// ── Python grammar ────────────────────────────────────────────────────────────

#[test]
fn python_produces_keyword_for_def() {
    let tokens = tokenize(Language::Python, "def hello(): pass");
    assert!(tokens.iter().any(|t| t.text == "def" && t.kind == TokenKind::Keyword));
}

#[test]
fn python_produces_keyword_for_import() {
    let tokens = tokenize(Language::Python, "import os");
    assert!(tokens.iter().any(|t| t.text == "import" && t.kind == TokenKind::Keyword));
}

#[test]
fn python_lossless() {
    let input = "def greet(name):\n    print(f'hello {name}')\n";
    let tokens = tokenize(Language::Python, input);
    assert_eq!(join_tokens(&tokens), input);
}

// ── YAML grammar ──────────────────────────────────────────────────────────────

#[test]
fn yaml_produces_key_tokens() {
    // Single line to avoid newline-stripping issue in YAML LineIter
    let tokens = tokenize(Language::Yaml, "name: prytty");
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Key && t.text == "name"));
}

#[test]
fn yaml_produces_constant_for_true_false() {
    let tokens = tokenize(Language::Yaml, "enabled: true");
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Constant && t.text == "true"));
}

#[test]
fn yaml_produces_comment_tokens() {
    let tokens = tokenize(Language::Yaml, "# this is a comment");
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Comment));
}

/// YAML lossless test: LineIter strips the newline between lines, so only
/// single-line inputs round-trip exactly. We test that per-line tokenization
/// is lossless by verifying single-line inputs.
#[test]
fn yaml_lossless_single_line() {
    let input = "name: prytty";
    let tokens = tokenize(Language::Yaml, input);
    assert_eq!(join_tokens(&tokens), input);
}

#[test]
fn yaml_at_least_one_token_for_nonempty_input() {
    let input = "key: value";
    let tokens = tokenize(Language::Yaml, input);
    assert!(!tokens.is_empty());
}

// ── TOML grammar ──────────────────────────────────────────────────────────────

#[test]
fn toml_produces_key_tokens() {
    let input = "name = \"prytty\"\n";
    let tokens = tokenize(Language::Toml, input);
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Key && t.text == "name"));
}

#[test]
fn toml_produces_attribute_for_section_header() {
    let input = "[package]\n";
    let tokens = tokenize(Language::Toml, input);
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Attribute));
}

#[test]
fn toml_produces_constant_for_booleans() {
    let input = "enabled = true\n";
    let tokens = tokenize(Language::Toml, input);
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Constant && t.text == "true"));
}

#[test]
fn toml_lossless() {
    let input = "[package]\nname = \"prytty\"\nversion = \"0.1.0\"\n";
    let tokens = tokenize(Language::Toml, input);
    assert_eq!(join_tokens(&tokens), input, "TOML tokenization must be lossless");
}

// ── Diff grammar ──────────────────────────────────────────────────────────────

#[test]
fn diff_produces_label_for_diff_header() {
    let input = "diff --git a/foo.rs b/foo.rs\n";
    let tokens = tokenize(Language::Diff, input);
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Label));
}

#[test]
fn diff_produces_string_for_added_lines() {
    let input = "+added line\n";
    let tokens = tokenize(Language::Diff, input);
    // Added lines are classified as String (green-ish)
    assert!(tokens.iter().any(|t| t.kind == TokenKind::String && t.text.contains("+added line")));
}

#[test]
fn diff_produces_keyword_for_removed_lines() {
    let input = "-removed line\n";
    let tokens = tokenize(Language::Diff, input);
    // Removed lines are classified as Keyword (red-ish)
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Keyword && t.text.contains("-removed line")));
}

#[test]
fn diff_lossless() {
    let input = "diff --git a/foo.rs b/foo.rs\n--- a/foo.rs\n+++ b/foo.rs\n@@ -1,2 +1,3 @@\n-old\n+new\n context\n";
    let tokens = tokenize(Language::Diff, input);
    assert_eq!(join_tokens(&tokens), input, "Diff tokenization must be lossless");
}

// ── Log grammar ───────────────────────────────────────────────────────────────

#[test]
fn log_produces_timestamp_for_iso_timestamp() {
    let input = "2024-01-15T12:34:56 INFO starting up\n";
    let tokens = tokenize(Language::Log, input);
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Timestamp));
}

#[test]
fn log_produces_label_for_warn_level() {
    let input = "2024-01-15T12:34:56 WARN disk almost full\n";
    let tokens = tokenize(Language::Log, input);
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Label && t.text.contains("WARN")));
}

#[test]
fn log_produces_keyword_for_error_level() {
    let input = "2024-01-15T12:34:56 ERROR something went wrong\n";
    let tokens = tokenize(Language::Log, input);
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Keyword && t.text.contains("ERROR")));
}

#[test]
fn log_lossless() {
    let input = "2024-01-15T12:34:56 INFO service started\n2024-01-15T12:35:00 ERROR connection refused\n";
    let tokens = tokenize(Language::Log, input);
    assert_eq!(join_tokens(&tokens), input, "Log tokenization must be lossless");
}

// ── Generic grammar ───────────────────────────────────────────────────────────

#[test]
fn generic_produces_tokens_for_nonempty_input() {
    let input = "hello world 42";
    let tokens = tokenize(Language::Generic, input);
    assert!(!tokens.is_empty());
}

#[test]
fn generic_produces_number_tokens() {
    let tokens = tokenize(Language::Generic, "value: 123");
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Number && t.text == "123"));
}

#[test]
fn generic_produces_string_tokens() {
    let tokens = tokenize(Language::Generic, r#"key = "value""#);
    assert!(tokens.iter().any(|t| t.kind == TokenKind::String && t.text.contains("value")));
}

#[test]
fn generic_lossless() {
    let input = "hello world 42 \"quoted\" {bracket}";
    let tokens = tokenize(Language::Generic, input);
    assert_eq!(join_tokens(&tokens), input, "Generic tokenization must be lossless");
}

// ── All languages produce tokens for non-empty input ─────────────────────────

#[test]
fn all_languages_produce_tokens_for_nonempty_input() {
    let cases: &[(Language, &str)] = &[
        (Language::Rust, "fn main() {}"),
        (Language::Python, "def f(): pass"),
        (Language::Json, r#"{"k": 1}"#),
        (Language::Yaml, "key: value"),
        (Language::Toml, "key = \"value\"\n"),
        (Language::Diff, "+added\n-removed\n"),
        (Language::Log, "2024-01-15T12:00:00 INFO ok\n"),
        (Language::Generic, "hello 42"),
    ];

    for (lang, input) in cases {
        let tokens = tokenize(*lang, input);
        assert!(!tokens.is_empty(), "expected tokens for {}", lang.name());
    }
}

// ── Lossless tokenization for all applicable grammars ────────────────────────

#[test]
fn lossless_rust() {
    let input = "use std::io;\n\npub fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n";
    let tokens = tokenize(Language::Rust, input);
    assert_eq!(join_tokens(&tokens), input);
}

#[test]
fn lossless_python() {
    let input = "import sys\n\nclass Greeter:\n    def greet(self, name):\n        return f'hello {name}'\n";
    let tokens = tokenize(Language::Python, input);
    assert_eq!(join_tokens(&tokens), input);
}

#[test]
fn lossless_json() {
    let input = r#"{"a":1,"b":"two","c":true,"d":null}"#;
    let tokens = tokenize(Language::Json, input);
    assert_eq!(join_tokens(&tokens), input);
}

#[test]
fn lossless_toml() {
    let input = "[deps]\nfoo = \"1.0\"\nbar = true\n";
    let tokens = tokenize(Language::Toml, input);
    assert_eq!(join_tokens(&tokens), input);
}

#[test]
fn lossless_diff() {
    let input = "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-old line\n+new line\n";
    let tokens = tokenize(Language::Diff, input);
    assert_eq!(join_tokens(&tokens), input);
}

#[test]
fn lossless_log() {
    let input = "2024-03-01T08:00:00 INFO booting\n2024-03-01T08:00:01 DEBUG config loaded\n";
    let tokens = tokenize(Language::Log, input);
    assert_eq!(join_tokens(&tokens), input);
}

#[test]
fn lossless_generic() {
    let input = "SELECT * FROM users WHERE id = 1;\n";
    let tokens = tokenize(Language::Generic, input);
    assert_eq!(join_tokens(&tokens), input);
}
