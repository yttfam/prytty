use prytty_core::{detect_language, strip_ansi, AnsiWriter, ColorMode, Language};
use prytty_syntax::tokenize;
use wasm_bindgen::prelude::*;

/// Highlight text, return ANSI-colored string.
/// language: optional hint ("rust", "json", "diff", etc.)
/// If None/undefined, auto-detect from content.
///
/// NOTE: This expects clean text. If input contains pre-existing ANSI
/// escape sequences (e.g., raw PTY output), call `sanitize()` first
/// to strip them, or use `highlight_safe()` which does both.
#[wasm_bindgen]
pub fn highlight(text: &str, language: Option<String>) -> String {
    let lang = language
        .as_deref()
        .and_then(Language::from_name)
        .unwrap_or_else(|| detect_language(None, text));

    let tokens = tokenize(lang, text);
    // Always truecolor — Crytter's VTE handles all SGR sequences
    let writer = AnsiWriter::new(ColorMode::TrueColor, Default::default());
    writer.render(&tokens)
}

/// Strip ANSI escapes then highlight. Safe for raw PTY output.
#[wasm_bindgen]
pub fn highlight_safe(text: &str, language: Option<String>) -> String {
    let clean = strip_ansi(text);
    let lang = language
        .as_deref()
        .and_then(Language::from_name)
        .unwrap_or_else(|| detect_language(None, &clean));

    let tokens = tokenize(lang, &clean);
    let writer = AnsiWriter::new(ColorMode::TrueColor, Default::default());
    writer.render(&tokens)
}

/// Strip all ANSI escape sequences from text.
/// Use this to clean raw PTY output before highlighting.
#[wasm_bindgen]
pub fn sanitize(text: &str) -> String {
    strip_ansi(text)
}

/// Detect the language of the given text. Returns the language name
/// or "generic" if no match.
#[wasm_bindgen]
pub fn detect(text: &str) -> String {
    detect_language(None, text).name().to_string()
}

/// List all supported language names.
#[wasm_bindgen]
pub fn languages() -> Vec<String> {
    vec![
        "rust".into(),
        "python".into(),
        "json".into(),
        "yaml".into(),
        "toml".into(),
        "diff".into(),
        "log".into(),
        "generic".into(),
    ]
}
