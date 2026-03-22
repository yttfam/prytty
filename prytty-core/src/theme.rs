use crate::token::TokenKind;

/// RGB color + text style for a token kind.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub fg: (u8, u8, u8),
    pub bold: bool,
    pub italic: bool,
}

impl Style {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            fg: (r, g, b),
            bold: false,
            italic: false,
        }
    }

    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
}

/// A color theme mapping token kinds to styles.
/// Default is a VS Code Dark+ inspired palette (looks good on #1e1e1e backgrounds).
#[derive(Debug, Clone)]
pub struct Theme {
    pub keyword: Style,
    pub type_: Style,
    pub function: Style,
    pub string: Style,
    pub number: Style,
    pub comment: Style,
    pub operator: Style,
    pub punctuation: Style,
    pub variable: Style,
    pub constant: Style,
    pub attribute: Style,
    pub builtin: Style,
    pub label: Style,
    pub key: Style,
    pub escape: Style,
    pub url: Style,
    pub path: Style,
    pub ip: Style,
    pub timestamp: Style,
    pub plain: Style,
}

impl Default for Theme {
    fn default() -> Self {
        // VS Code Dark+ inspired
        Self {
            keyword: Style::new(197, 134, 192).bold(),   // purple
            type_: Style::new(78, 201, 176),              // teal
            function: Style::new(220, 220, 170),          // light yellow
            string: Style::new(206, 145, 120),            // orange-brown
            number: Style::new(181, 206, 168),            // light green
            comment: Style::new(106, 153, 85).italic(),   // green, dim
            operator: Style::new(212, 212, 212),          // light gray
            punctuation: Style::new(150, 150, 150),       // gray
            variable: Style::new(156, 220, 254),          // light blue
            constant: Style::new(100, 150, 224),          // blue
            attribute: Style::new(156, 220, 254),         // light blue
            builtin: Style::new(78, 201, 176),            // teal
            label: Style::new(220, 220, 170).bold(),      // yellow, bold
            key: Style::new(156, 220, 254),               // light blue
            escape: Style::new(215, 186, 125),            // gold
            url: Style::new(100, 150, 224).italic(),      // blue, underline-ish
            path: Style::new(156, 220, 254),              // light blue
            ip: Style::new(181, 206, 168),                // light green
            timestamp: Style::new(106, 153, 85),          // dim green
            plain: Style::new(212, 212, 212),             // default fg
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_keyword_is_bold() {
        let theme = Theme::default();
        assert!(theme.keyword.bold, "keyword should be bold");
    }

    #[test]
    fn default_theme_comment_is_italic() {
        let theme = Theme::default();
        assert!(theme.comment.italic, "comment should be italic");
    }

    #[test]
    fn default_theme_all_fg_colors_nonzero() {
        let theme = Theme::default();
        // Every style should have a non-black foreground (i.e. intentional color)
        let styles = [
            theme.keyword, theme.type_, theme.function, theme.string,
            theme.number, theme.comment, theme.operator, theme.punctuation,
            theme.variable, theme.constant, theme.attribute, theme.builtin,
            theme.label, theme.key, theme.escape, theme.url, theme.path,
            theme.ip, theme.timestamp, theme.plain,
        ];
        for style in &styles {
            let (r, g, b) = style.fg;
            assert!(r > 0 || g > 0 || b > 0, "style has zero fg color: {style:?}");
        }
    }

    #[test]
    fn style_for_maps_all_token_kinds() {
        use crate::token::TokenKind;
        let theme = Theme::default();
        let kinds = [
            TokenKind::Keyword, TokenKind::Type, TokenKind::Function,
            TokenKind::String, TokenKind::Number, TokenKind::Comment,
            TokenKind::Operator, TokenKind::Punctuation, TokenKind::Variable,
            TokenKind::Constant, TokenKind::Attribute, TokenKind::Builtin,
            TokenKind::Label, TokenKind::Key, TokenKind::Escape,
            TokenKind::Url, TokenKind::Path, TokenKind::Ip,
            TokenKind::Timestamp, TokenKind::Plain,
        ];
        for kind in &kinds {
            // Just ensure it doesn't panic and returns some style
            let _style = theme.style_for(*kind);
        }
    }

    #[test]
    fn style_for_keyword_matches_theme_keyword() {
        use crate::token::TokenKind;
        let theme = Theme::default();
        let s = theme.style_for(TokenKind::Keyword);
        assert_eq!(s.fg, theme.keyword.fg);
        assert_eq!(s.bold, theme.keyword.bold);
    }

    #[test]
    fn style_for_string_matches_theme_string() {
        use crate::token::TokenKind;
        let theme = Theme::default();
        let s = theme.style_for(TokenKind::String);
        assert_eq!(s.fg, theme.string.fg);
    }

    #[test]
    fn style_new_defaults_bold_italic_false() {
        let s = Style::new(100, 150, 200);
        assert!(!s.bold);
        assert!(!s.italic);
        assert_eq!(s.fg, (100, 150, 200));
    }

    #[test]
    fn style_bold_sets_bold() {
        let s = Style::new(100, 150, 200).bold();
        assert!(s.bold);
        assert!(!s.italic);
    }

    #[test]
    fn style_italic_sets_italic() {
        let s = Style::new(100, 150, 200).italic();
        assert!(s.italic);
        assert!(!s.bold);
    }
}

impl Theme {
    pub fn style_for(&self, kind: TokenKind) -> Style {
        match kind {
            TokenKind::Keyword => self.keyword,
            TokenKind::Type => self.type_,
            TokenKind::Function => self.function,
            TokenKind::String => self.string,
            TokenKind::Number => self.number,
            TokenKind::Comment => self.comment,
            TokenKind::Operator => self.operator,
            TokenKind::Punctuation => self.punctuation,
            TokenKind::Variable => self.variable,
            TokenKind::Constant => self.constant,
            TokenKind::Attribute => self.attribute,
            TokenKind::Builtin => self.builtin,
            TokenKind::Label => self.label,
            TokenKind::Key => self.key,
            TokenKind::Escape => self.escape,
            TokenKind::Url => self.url,
            TokenKind::Path => self.path,
            TokenKind::Ip => self.ip,
            TokenKind::Timestamp => self.timestamp,
            TokenKind::Plain => self.plain,
        }
    }
}
