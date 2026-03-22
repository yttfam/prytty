use crate::theme::{Style, Theme};
use crate::token::{Token, TokenKind};
use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    TrueColor,
    Color256,
    Color16,
    None,
}

impl ColorMode {
    /// Detect the best color mode from the environment.
    pub fn detect() -> Self {
        if let Ok(ct) = std::env::var("COLORTERM") {
            if ct == "truecolor" || ct == "24bit" {
                return Self::TrueColor;
            }
        }
        if let Ok(term) = std::env::var("TERM") {
            if term.contains("256color") {
                return Self::Color256;
            }
        }
        Self::Color16
    }
}

/// Writes ANSI-colored output from a token stream.
pub struct AnsiWriter {
    pub mode: ColorMode,
    pub theme: Theme,
}

impl AnsiWriter {
    pub fn new(mode: ColorMode, theme: Theme) -> Self {
        Self { mode, theme }
    }

    pub fn auto() -> Self {
        Self::new(ColorMode::detect(), Theme::default())
    }

    /// Render tokens to an ANSI-colored string.
    pub fn render(&self, tokens: &[Token<'_>]) -> String {
        let mut out = String::new();
        for token in tokens {
            self.write_token(&mut out, token);
        }
        if !tokens.is_empty() {
            out.push_str("\x1b[0m");
        }
        out
    }

    fn write_token(&self, out: &mut String, token: &Token<'_>) {
        let style = self.theme.style_for(token.kind);
        if self.mode == ColorMode::None || token.kind == TokenKind::Plain {
            out.push_str(token.text);
            return;
        }
        self.write_sgr(out, &style);
        out.push_str(token.text);
        out.push_str("\x1b[0m");
    }

    fn write_sgr(&self, out: &mut String, style: &Style) {
        let (r, g, b) = style.fg;
        match self.mode {
            ColorMode::TrueColor => {
                let _ = write!(out, "\x1b[38;2;{r};{g};{b}m");
            }
            ColorMode::Color256 => {
                let idx = rgb_to_256(r, g, b);
                let _ = write!(out, "\x1b[38;5;{idx}m");
            }
            ColorMode::Color16 => {
                let code = rgb_to_16(r, g, b);
                let _ = write!(out, "\x1b[{code}m");
            }
            ColorMode::None => {}
        }
        if style.bold {
            out.push_str("\x1b[1m");
        }
        if style.italic {
            out.push_str("\x1b[3m");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Token, TokenKind};
    use crate::theme::Theme;
    use std::sync::Mutex;

    // Serialize tests that mutate env vars to avoid race conditions.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    // --- ColorMode::detect ---

    #[test]
    fn detect_truecolor_from_colorterm() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("COLORTERM", "truecolor");
            std::env::remove_var("TERM");
        }
        let mode = ColorMode::detect();
        unsafe { std::env::remove_var("COLORTERM"); }
        assert_eq!(mode, ColorMode::TrueColor);
    }

    #[test]
    fn detect_truecolor_24bit() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("COLORTERM", "24bit");
            std::env::remove_var("TERM");
        }
        let mode = ColorMode::detect();
        unsafe { std::env::remove_var("COLORTERM"); }
        assert_eq!(mode, ColorMode::TrueColor);
    }

    #[test]
    fn detect_256color_from_term() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::remove_var("COLORTERM");
            std::env::set_var("TERM", "xterm-256color");
        }
        let mode = ColorMode::detect();
        unsafe { std::env::remove_var("TERM"); }
        assert_eq!(mode, ColorMode::Color256);
    }

    #[test]
    fn detect_16color_fallback() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::remove_var("COLORTERM");
            std::env::set_var("TERM", "xterm");
        }
        let mode = ColorMode::detect();
        unsafe { std::env::remove_var("TERM"); }
        assert_eq!(mode, ColorMode::Color16);
    }

    // --- rgb_to_256 ---

    #[test]
    fn rgb_to_256_near_black_grayscale() {
        // r=g=b < 8 → returns 16
        assert_eq!(rgb_to_256(0, 0, 0), 16);
        assert_eq!(rgb_to_256(5, 5, 5), 16);
    }

    #[test]
    fn rgb_to_256_near_white_grayscale() {
        // r > 248 → returns 231
        assert_eq!(rgb_to_256(255, 255, 255), 231);
        assert_eq!(rgb_to_256(249, 249, 249), 231);
    }

    #[test]
    fn rgb_to_256_mid_grayscale() {
        // mid-gray, r≈g≈b, 8 <= r <= 248
        let idx = rgb_to_256(128, 128, 128);
        // Should be in the grayscale ramp range 232..=255
        assert!(idx >= 232, "expected grayscale ramp, got {idx}");
    }

    #[test]
    fn rgb_to_256_color_cube_pure_red() {
        // Pure red: max-min is large, goes to color cube
        let idx = rgb_to_256(255, 0, 0);
        // ri=5, gi=0, bi=0 → 16 + 36*5 + 0 + 0 = 196
        assert_eq!(idx, 196);
    }

    #[test]
    fn rgb_to_256_color_cube_pure_blue() {
        let idx = rgb_to_256(0, 0, 255);
        // ri=0, gi=0, bi=5 → 16 + 0 + 0 + 5 = 21
        assert_eq!(idx, 21);
    }

    // --- rgb_to_16 ---

    #[test]
    fn rgb_to_16_black() {
        assert_eq!(rgb_to_16(0, 0, 0), 30); // black, dim
    }

    #[test]
    fn rgb_to_16_red() {
        assert_eq!(rgb_to_16(200, 0, 0), 31); // red, dim
    }

    #[test]
    fn rgb_to_16_green() {
        assert_eq!(rgb_to_16(0, 200, 0), 32); // green, dim
    }

    #[test]
    fn rgb_to_16_white_bright() {
        // r=g=b=255: sum=765 > 384 → bright, base=37 → 97
        assert_eq!(rgb_to_16(255, 255, 255), 97);
    }

    #[test]
    fn rgb_to_16_blue_dim() {
        assert_eq!(rgb_to_16(0, 0, 200), 34); // blue, dim (sum=200 <= 384)
    }

    #[test]
    fn rgb_to_16_bright_red() {
        // r=255, g=50, b=50: sum=355 <= 384, so base=31, not bright → 31
        // Actually: (255+50+50)=355 <= 384 → dim
        // r>128, g<128, b<128 → 31
        assert_eq!(rgb_to_16(255, 50, 50), 31);
    }

    // --- AnsiWriter rendering ---

    fn keyword_token() -> Token<'static> {
        Token { text: "fn", kind: TokenKind::Keyword }
    }

    fn plain_token() -> Token<'static> {
        Token { text: "foo", kind: TokenKind::Plain }
    }

    #[test]
    fn render_empty_tokens_produces_empty_string() {
        let writer = AnsiWriter::new(ColorMode::TrueColor, Theme::default());
        assert_eq!(writer.render(&[]), "");
    }

    #[test]
    fn render_none_mode_strips_ansi() {
        let writer = AnsiWriter::new(ColorMode::None, Theme::default());
        let tokens = vec![keyword_token()];
        let out = writer.render(&tokens);
        // None mode: no escape sequences except reset at end
        assert!(out.contains("fn"));
        assert!(!out.contains("\x1b[38;"), "unexpected color code in None mode");
    }

    #[test]
    fn render_plain_token_no_color_code() {
        let writer = AnsiWriter::new(ColorMode::TrueColor, Theme::default());
        let tokens = vec![plain_token()];
        let out = writer.render(&tokens);
        // Plain tokens skip SGR in write_token
        assert!(out.contains("foo"));
        assert!(!out.contains("\x1b[38;"), "plain token should not get color SGR");
    }

    #[test]
    fn render_truecolor_sgr_format() {
        let writer = AnsiWriter::new(ColorMode::TrueColor, Theme::default());
        let tokens = vec![keyword_token()];
        let out = writer.render(&tokens);
        // Expect ESC[38;2;R;G;Bm format
        assert!(out.contains("\x1b[38;2;"), "expected truecolor SGR");
    }

    #[test]
    fn render_256_color_sgr_format() {
        let writer = AnsiWriter::new(ColorMode::Color256, Theme::default());
        let tokens = vec![keyword_token()];
        let out = writer.render(&tokens);
        assert!(out.contains("\x1b[38;5;"), "expected 256-color SGR");
    }

    #[test]
    fn render_16_color_sgr_format() {
        let writer = AnsiWriter::new(ColorMode::Color16, Theme::default());
        let tokens = vec![keyword_token()];
        let out = writer.render(&tokens);
        // Should have ESC[<code>m where code is 30-37 or 90-97
        assert!(out.contains("\x1b["), "expected 16-color SGR");
    }

    #[test]
    fn render_ends_with_reset() {
        let writer = AnsiWriter::new(ColorMode::TrueColor, Theme::default());
        let tokens = vec![keyword_token()];
        let out = writer.render(&tokens);
        assert!(out.ends_with("\x1b[0m"), "output should end with reset");
    }

    #[test]
    fn render_bold_token_includes_bold_code() {
        // Keyword style has bold=true in default theme
        let writer = AnsiWriter::new(ColorMode::TrueColor, Theme::default());
        let tokens = vec![keyword_token()];
        let out = writer.render(&tokens);
        assert!(out.contains("\x1b[1m"), "expected bold escape for keyword");
    }
}

/// Approximate RGB to 256-color palette.
fn rgb_to_256(r: u8, g: u8, b: u8) -> u8 {
    // Use grayscale ramp if r ≈ g ≈ b
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
    // Map to 6x6x6 color cube
    let ri = (r as u16 * 5 / 255) as u8;
    let gi = (g as u16 * 5 / 255) as u8;
    let bi = (b as u16 * 5 / 255) as u8;
    16 + 36 * ri + 6 * gi + bi
}

/// Very rough RGB to 16-color ANSI.
fn rgb_to_16(r: u8, g: u8, b: u8) -> u8 {
    let bright = (r as u16 + g as u16 + b as u16) > 384;
    let base = match (r > 128, g > 128, b > 128) {
        (false, false, false) => 30, // black
        (true, false, false) => 31,  // red
        (false, true, false) => 32,  // green
        (true, true, false) => 33,   // yellow
        (false, false, true) => 34,  // blue
        (true, false, true) => 35,   // magenta
        (false, true, true) => 36,   // cyan
        (true, true, true) => 37,    // white
    };
    if bright { base + 60 } else { base }
}
