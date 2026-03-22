mod color;
mod detect;
mod sanitize;
#[cfg(feature = "syntect")]
mod syntect_hl;
mod theme;
mod token;

pub use color::{AnsiWriter, ColorMode};
pub use detect::{detect_language, Language};
pub use sanitize::strip_ansi;
#[cfg(feature = "syntect")]
pub use syntect_hl::SyntectHighlighter;
pub use theme::{Style, Theme};
pub use token::{Token, TokenKind};
