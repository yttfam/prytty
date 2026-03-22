use prytty_core::{AnsiWriter, ColorMode, Language, Theme, TokenKind, Token, detect_language};
use std::path::Path;

// ── detect_language ──────────────────────────────────────────────────────────

#[test]
fn detect_by_rs_extension() {
    let path = Path::new("main.rs");
    assert_eq!(detect_language(Some(path), ""), Language::Rust);
}

#[test]
fn detect_by_py_extension() {
    let path = Path::new("script.py");
    assert_eq!(detect_language(Some(path), ""), Language::Python);
}

#[test]
fn detect_by_json_extension() {
    let path = Path::new("data.json");
    assert_eq!(detect_language(Some(path), ""), Language::Json);
}

#[test]
fn detect_by_yaml_extension() {
    let path = Path::new("config.yaml");
    assert_eq!(detect_language(Some(path), ""), Language::Yaml);
}

#[test]
fn detect_by_yml_extension() {
    let path = Path::new("docker-compose.yml");
    assert_eq!(detect_language(Some(path), ""), Language::Yaml);
}

#[test]
fn detect_by_toml_extension() {
    let path = Path::new("Cargo.toml");
    assert_eq!(detect_language(Some(path), ""), Language::Toml);
}

#[test]
fn detect_by_diff_extension() {
    let path = Path::new("patch.diff");
    assert_eq!(detect_language(Some(path), ""), Language::Diff);
}

#[test]
fn detect_by_patch_extension() {
    let path = Path::new("fix.patch");
    assert_eq!(detect_language(Some(path), ""), Language::Diff);
}

#[test]
fn detect_by_log_extension() {
    let path = Path::new("app.log");
    assert_eq!(detect_language(Some(path), ""), Language::Log);
}

#[test]
fn detect_python_shebang() {
    let content = "#!/usr/bin/env python3\nprint('hello')";
    assert_eq!(detect_language(None, content), Language::Python);
}

#[test]
fn detect_json_from_content_object() {
    let content = r#"{"name": "prytty", "version": "0.1.0"}"#;
    assert_eq!(detect_language(None, content), Language::Json);
}

#[test]
fn detect_json_from_content_array() {
    let content = r#"[1, 2, 3]"#;
    assert_eq!(detect_language(None, content), Language::Json);
}

#[test]
fn detect_diff_from_content() {
    let content = "diff --git a/foo.rs b/foo.rs\n--- a/foo.rs\n+++ b/foo.rs\n";
    assert_eq!(detect_language(None, content), Language::Diff);
}

#[test]
fn detect_yaml_from_content() {
    let content = "---\nname: prytty\n";
    assert_eq!(detect_language(None, content), Language::Yaml);
}

#[test]
fn detect_rust_from_content() {
    let content = "fn main() {\n    let x = 42;\n    println!(\"{}\", x);\n}\n";
    assert_eq!(detect_language(None, content), Language::Rust);
}

#[test]
fn detect_python_from_content() {
    let content = "def hello():\n    print('world')\n\nif __name__ == '__main__':\n    hello()\n";
    assert_eq!(detect_language(None, content), Language::Python);
}

#[test]
fn detect_log_from_content() {
    let content = "2024-01-15T12:00:00 INFO starting\n2024-01-15T12:00:01 DEBUG ready\n";
    assert_eq!(detect_language(None, content), Language::Log);
}

#[test]
fn detect_no_path_falls_back_to_content() {
    // No path, trivially recognizable JSON
    let content = r#"{"key": "value"}"#;
    assert_eq!(detect_language(None, content), Language::Json);
}

// ── AnsiWriter::render ────────────────────────────────────────────────────────

fn make_tokens(texts: &[(&'static str, TokenKind)]) -> Vec<Token<'static>> {
    texts.iter().map(|(t, k)| Token { text: t, kind: *k }).collect()
}

#[test]
fn render_empty_tokens_produces_empty_string() {
    let writer = AnsiWriter::new(ColorMode::TrueColor, Theme::default());
    let output = writer.render(&[]);
    assert!(output.is_empty());
}

#[test]
fn render_truecolor_contains_ansi_sequences() {
    let writer = AnsiWriter::new(ColorMode::TrueColor, Theme::default());
    let tokens = make_tokens(&[("fn", TokenKind::Keyword)]);
    let output = writer.render(&tokens);
    assert!(output.contains("\x1b["), "expected ANSI escape in: {:?}", output);
}

#[test]
fn render_color256_contains_ansi_sequences() {
    let writer = AnsiWriter::new(ColorMode::Color256, Theme::default());
    let tokens = make_tokens(&[("42", TokenKind::Number)]);
    let output = writer.render(&tokens);
    assert!(output.contains("\x1b["), "expected ANSI escape in: {:?}", output);
}

#[test]
fn render_color16_contains_ansi_sequences() {
    let writer = AnsiWriter::new(ColorMode::Color16, Theme::default());
    let tokens = make_tokens(&[("hello", TokenKind::String)]);
    let output = writer.render(&tokens);
    assert!(output.contains("\x1b["), "expected ANSI escape in: {:?}", output);
}

#[test]
fn render_none_mode_no_color_sequences() {
    let writer = AnsiWriter::new(ColorMode::None, Theme::default());
    let tokens = make_tokens(&[("fn", TokenKind::Keyword), (" ", TokenKind::Plain), ("main", TokenKind::Function)]);
    let output = writer.render(&tokens);
    // ColorMode::None suppresses color SGR codes but render() still appends the
    // final reset \x1b[0m when tokens are present. Check no color codes appear.
    assert!(!output.contains("\x1b[38;"), "unexpected color SGR in None mode: {:?}", output);
    assert!(!output.contains("\x1b[1m"), "unexpected bold in None mode: {:?}", output);
    assert!(output.contains("fn"));
    assert!(output.contains("main"));
}

#[test]
fn render_ends_with_reset_when_tokens_present() {
    let writer = AnsiWriter::new(ColorMode::TrueColor, Theme::default());
    let tokens = make_tokens(&[("fn", TokenKind::Keyword)]);
    let output = writer.render(&tokens);
    assert!(output.ends_with("\x1b[0m"), "output should end with reset: {:?}", output);
}

#[test]
fn render_ends_with_reset_color256() {
    let writer = AnsiWriter::new(ColorMode::Color256, Theme::default());
    let tokens = make_tokens(&[("struct", TokenKind::Keyword), (" ", TokenKind::Plain), ("Foo", TokenKind::Type)]);
    let output = writer.render(&tokens);
    assert!(output.ends_with("\x1b[0m"), "output should end with reset: {:?}", output);
}

#[test]
fn render_plain_tokens_no_color_sequence_in_none_mode() {
    let writer = AnsiWriter::new(ColorMode::None, Theme::default());
    let tokens = make_tokens(&[("hello world", TokenKind::Plain)]);
    let output = writer.render(&tokens);
    // No color SGR codes, text is present. The final reset is still appended.
    assert!(output.contains("hello world"));
    assert!(!output.contains("\x1b[38;"), "no color codes in None mode");
}

#[test]
fn render_multiple_token_kinds_truecolor() {
    let writer = AnsiWriter::new(ColorMode::TrueColor, Theme::default());
    let tokens = make_tokens(&[
        ("fn", TokenKind::Keyword),
        (" ", TokenKind::Plain),
        ("add", TokenKind::Function),
        ("(", TokenKind::Punctuation),
        ("x", TokenKind::Variable),
        (")", TokenKind::Punctuation),
    ]);
    let output = writer.render(&tokens);
    // Should contain the original text
    assert!(output.contains("fn"));
    assert!(output.contains("add"));
    assert!(output.contains("("));
    // Should have ANSI escapes
    assert!(output.contains("\x1b["));
    // Should end with reset
    assert!(output.ends_with("\x1b[0m"));
}

// ── Full pipeline: detect → tokenize (via prytty_syntax) → render ─────────────

#[test]
fn full_pipeline_rust() {
    use prytty_syntax::tokenize;

    let code = "fn main() {\n    let x = 42;\n}\n";
    let lang = detect_language(None, code);
    assert_eq!(lang, Language::Rust);

    let tokens = tokenize(lang, code);
    assert!(!tokens.is_empty());

    let writer = AnsiWriter::new(ColorMode::TrueColor, Theme::default());
    let output = writer.render(&tokens);

    assert!(output.contains("\x1b["));
    assert!(output.ends_with("\x1b[0m"));
    // Text content is preserved (strip ANSI to check)
    let stripped = strip_ansi(&output);
    assert!(stripped.contains("fn"));
    assert!(stripped.contains("main"));
    assert!(stripped.contains("42"));
}

#[test]
fn full_pipeline_json() {
    use prytty_syntax::tokenize;

    let code = r#"{"name": "prytty", "count": 3}"#;
    let lang = detect_language(None, code);
    assert_eq!(lang, Language::Json);

    let tokens = tokenize(lang, code);
    assert!(!tokens.is_empty());

    let writer = AnsiWriter::new(ColorMode::TrueColor, Theme::default());
    let output = writer.render(&tokens);

    assert!(output.ends_with("\x1b[0m"));
    let stripped = strip_ansi(&output);
    assert!(stripped.contains("name"));
    assert!(stripped.contains("prytty"));
    assert!(stripped.contains("3"));
}

#[test]
fn full_pipeline_none_mode_is_plain_text() {
    use prytty_syntax::tokenize;

    let code = r#"{"key": "val"}"#;
    let lang = detect_language(None, code);
    let tokens = tokenize(lang, code);

    let writer = AnsiWriter::new(ColorMode::None, Theme::default());
    let output = writer.render(&tokens);

    // ColorMode::None suppresses color SGR codes but appends final reset.
    assert!(!output.contains("\x1b[38;"), "none mode should have no color SGR codes");
    assert!(!output.contains("\x1b[1m"), "none mode should have no bold codes");

    // The concatenated text of all tokens equals the original input (lossless).
    let joined: String = tokens.iter().map(|t| t.text).collect();
    assert_eq!(joined, code);
}

#[test]
fn full_pipeline_diff() {
    use prytty_syntax::tokenize;

    let code = "diff --git a/foo.rs b/foo.rs\n--- a/foo.rs\n+++ b/foo.rs\n@@ -1,2 +1,3 @@\n-old\n+new\n context\n";
    let lang = detect_language(None, code);
    assert_eq!(lang, Language::Diff);

    let tokens = tokenize(lang, code);
    assert!(!tokens.is_empty());

    let writer = AnsiWriter::new(ColorMode::TrueColor, Theme::default());
    let output = writer.render(&tokens);
    assert!(output.contains("\x1b["));
    assert!(output.ends_with("\x1b[0m"));
}

/// Strip ANSI escape sequences from a string.
fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // consume until 'm'
            for c in chars.by_ref() {
                if c == 'm' { break; }
            }
        } else {
            out.push(ch);
        }
    }
    out
}
