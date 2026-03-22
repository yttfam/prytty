//! Syntect-based syntax highlighting.
//! Gated behind the `syntect` feature flag.
//! Provides 200+ languages via Sublime Text grammars at the cost of ~4MB binary size.

use crate::ColorMode;
use std::fmt::Write;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// Cached syntax and theme sets. Expensive to create — reuse across calls.
pub struct SyntectHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl SyntectHighlighter {
    /// Create a new highlighter with default syntaxes and themes.
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    /// List available theme names.
    pub fn theme_names(&self) -> Vec<&str> {
        self.theme_set.themes.keys().map(|s| s.as_str()).collect()
    }

    /// List available syntax names.
    pub fn syntax_names(&self) -> Vec<&str> {
        self.syntax_set.syntaxes().iter().map(|s| s.name.as_str()).collect()
    }

    /// Find language by file extension (e.g., "rs", "py", "json").
    pub fn has_syntax_for_extension(&self, ext: &str) -> bool {
        self.syntax_set.find_syntax_by_extension(ext).is_some()
    }

    /// Highlight text, returning ANSI-colored string.
    /// `extension`: file extension hint (e.g., "rs", "py"). None for auto-detect.
    /// `theme`: syntect theme name (e.g., "base16-ocean.dark"). None for default.
    pub fn highlight(
        &self,
        text: &str,
        extension: Option<&str>,
        theme: Option<&str>,
        color_mode: ColorMode,
    ) -> String {
        let syntax = extension
            .and_then(|ext| self.syntax_set.find_syntax_by_extension(ext))
            .or_else(|| {
                text.lines()
                    .next()
                    .and_then(|first| self.syntax_set.find_syntax_by_first_line(first))
            })
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme_name = theme.unwrap_or("base16-ocean.dark");
        let theme = match self.theme_set.themes.get(theme_name) {
            Some(t) => t,
            None => {
                // Fallback to first available theme
                match self.theme_set.themes.values().next() {
                    Some(t) => t,
                    None => return text.to_string(),
                }
            }
        };

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut out = String::with_capacity(text.len() * 2);

        for line in LinesWithEndings::from(text) {
            let spans = match highlighter.highlight_line(line, &self.syntax_set) {
                Ok(s) => s,
                Err(_) => {
                    out.push_str(line);
                    continue;
                }
            };
            render_spans(&mut out, &spans, color_mode);
        }

        if !text.is_empty() {
            out.push_str("\x1b[0m");
        }
        out
    }
}

impl Default for SyntectHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// Render syntect spans to ANSI escape sequences.
fn render_spans(out: &mut String, spans: &[(Style, &str)], color_mode: ColorMode) {
    for (style, text) in spans {
        if color_mode == ColorMode::None {
            out.push_str(text);
            continue;
        }

        write_sgr(out, style, color_mode);
        out.push_str(text);
        out.push_str("\x1b[0m");
    }
}

fn write_sgr(out: &mut String, style: &Style, color_mode: ColorMode) {
    let fg = style.foreground;

    match color_mode {
        ColorMode::TrueColor => {
            let _ = write!(out, "\x1b[38;2;{};{};{}m", fg.r, fg.g, fg.b);
        }
        ColorMode::Color256 => {
            let idx = rgb_to_256(fg.r, fg.g, fg.b);
            let _ = write!(out, "\x1b[38;5;{idx}m");
        }
        ColorMode::Color16 => {
            let code = rgb_to_16(fg.r, fg.g, fg.b);
            let _ = write!(out, "\x1b[{code}m");
        }
        ColorMode::None => {}
    }

    if style.font_style.contains(FontStyle::BOLD) {
        out.push_str("\x1b[1m");
    }
    if style.font_style.contains(FontStyle::ITALIC) {
        out.push_str("\x1b[3m");
    }
    if style.font_style.contains(FontStyle::UNDERLINE) {
        out.push_str("\x1b[4m");
    }
}

// Reuse the same conversion functions from color.rs
fn rgb_to_256(r: u8, g: u8, b: u8) -> u8 {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    if max - min < 10 {
        if r < 8 {
            return 16;
        }
        if r > 248 {
            return 231;
        }
        return 232 + ((r as u16 - 8) * 24 / 240) as u8;
    }
    let ri = (r as u16 * 5 / 255) as u8;
    let gi = (g as u16 * 5 / 255) as u8;
    let bi = (b as u16 * 5 / 255) as u8;
    16 + 36 * ri + 6 * gi + bi
}

fn rgb_to_16(r: u8, g: u8, b: u8) -> u8 {
    let bright = (r as u16 + g as u16 + b as u16) > 384;
    let base = match (r > 128, g > 128, b > 128) {
        (false, false, false) => 30,
        (true, false, false) => 31,
        (false, true, false) => 32,
        (true, true, false) => 33,
        (false, false, true) => 34,
        (true, false, true) => 35,
        (false, true, true) => 36,
        (true, true, true) => 37,
    };
    if bright { base + 60 } else { base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_loads_without_panic() {
        let _hl = SyntectHighlighter::new();
    }

    #[test]
    fn has_rust_syntax() {
        let hl = SyntectHighlighter::new();
        assert!(hl.has_syntax_for_extension("rs"));
    }

    #[test]
    fn has_python_syntax() {
        let hl = SyntectHighlighter::new();
        assert!(hl.has_syntax_for_extension("py"));
    }

    #[test]
    fn highlight_rust_produces_ansi() {
        let hl = SyntectHighlighter::new();
        let out = hl.highlight("fn main() {}", Some("rs"), None, ColorMode::TrueColor);
        assert!(out.contains("\x1b[38;2;"));
        assert!(out.contains("fn"));
        assert!(out.ends_with("\x1b[0m"));
    }

    #[test]
    fn highlight_no_color_mode() {
        let hl = SyntectHighlighter::new();
        let out = hl.highlight("fn main() {}", Some("rs"), None, ColorMode::None);
        assert!(!out.contains("\x1b[38;"));
        assert!(out.contains("fn main()"));
    }

    #[test]
    fn highlight_unknown_extension_falls_back() {
        let hl = SyntectHighlighter::new();
        let out = hl.highlight("hello world", Some("xyznotreal"), None, ColorMode::TrueColor);
        assert!(out.contains("hello world"));
    }

    #[test]
    fn highlight_auto_detect_shebang() {
        let hl = SyntectHighlighter::new();
        let code = "#!/usr/bin/env python3\nprint('hello')";
        let out = hl.highlight(code, None, None, ColorMode::TrueColor);
        assert!(out.contains("\x1b["));
        assert!(out.contains("print"));
    }

    #[test]
    fn theme_names_not_empty() {
        let hl = SyntectHighlighter::new();
        assert!(!hl.theme_names().is_empty());
    }

    #[test]
    fn syntax_names_not_empty() {
        let hl = SyntectHighlighter::new();
        let names = hl.syntax_names();
        assert!(!names.is_empty());
        assert!(names.iter().any(|n| n.contains("Rust")));
    }

    #[test]
    fn highlight_256_color() {
        let hl = SyntectHighlighter::new();
        let out = hl.highlight("let x = 42;", Some("rs"), None, ColorMode::Color256);
        assert!(out.contains("\x1b[38;5;"));
    }
}
