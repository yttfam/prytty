use crate::generic::advance_char;
use crate::Grammar;
use prytty_core::{Token, TokenKind};

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(input: &str) -> Vec<(TokenKind, &str)> {
        LogGrammar.tokenize(input).into_iter().map(|t| (t.kind, t.text)).collect()
    }

    fn find_kind<'a>(tokens: &'a [(TokenKind, &'a str)], kind: TokenKind) -> Option<&'a str> {
        tokens.iter().find(|(k, _)| *k == kind).map(|(_, t)| *t)
    }

    fn has_kind(tokens: &[(TokenKind, &str)], kind: TokenKind) -> bool {
        tokens.iter().any(|(k, _)| *k == kind)
    }

    #[test]
    fn iso_timestamp_is_timestamp() {
        let tokens = tok("2024-01-15T12:34:56 INFO started");
        let ts = find_kind(&tokens, TokenKind::Timestamp);
        assert!(ts.is_some(), "expected a Timestamp token");
        assert!(ts.unwrap().starts_with("2024-01-15"), "timestamp should start with date");
    }

    #[test]
    fn iso_timestamp_space_separator() {
        let tokens = tok("2024-01-15 12:34:56 ERROR crash");
        assert!(has_kind(&tokens, TokenKind::Timestamp));
    }

    #[test]
    fn syslog_timestamp() {
        let tokens = tok("Jan 15 12:34:56 host sshd: session opened");
        assert!(has_kind(&tokens, TokenKind::Timestamp), "expected syslog timestamp");
    }

    #[test]
    fn error_level_is_keyword() {
        let tokens = tok("2024-01-15T12:00:00 ERROR something failed");
        assert!(has_kind(&tokens, TokenKind::Keyword), "ERROR should map to Keyword");
    }

    #[test]
    fn warn_level_is_label() {
        let tokens = tok("2024-01-15T12:00:00 WARN low disk");
        assert!(has_kind(&tokens, TokenKind::Label), "WARN should map to Label");
    }

    #[test]
    fn info_level_is_string() {
        let tokens = tok("2024-01-15T12:00:00 INFO starting up");
        assert!(has_kind(&tokens, TokenKind::String), "INFO should map to String");
    }

    #[test]
    fn debug_level_is_comment() {
        let tokens = tok("2024-01-15T12:00:00 DEBUG verbose output");
        assert!(has_kind(&tokens, TokenKind::Comment), "DEBUG should map to Comment");
    }

    #[test]
    fn lowercase_error_level() {
        let tokens = tok("2024-01-15T12:00:00 error bad thing");
        assert!(has_kind(&tokens, TokenKind::Keyword), "lowercase error should map to Keyword");
    }

    #[test]
    fn ipv4_address() {
        let tokens = tok("2024-01-15T12:00:00 INFO client 192.168.1.1 connected");
        assert!(has_kind(&tokens, TokenKind::Ip), "expected Ip token for IPv4 address");
        let ip = find_kind(&tokens, TokenKind::Ip).unwrap();
        assert_eq!(ip, "192.168.1.1");
    }

    #[test]
    fn url_http() {
        let tokens = tok("2024-01-15T12:00:00 INFO fetching http://example.com/path");
        assert!(has_kind(&tokens, TokenKind::Url), "expected Url token");
        let url = find_kind(&tokens, TokenKind::Url).unwrap();
        assert!(url.starts_with("http://"));
    }

    #[test]
    fn url_https() {
        let tokens = tok("2024-01-15T12:00:00 INFO fetching https://example.com");
        assert!(has_kind(&tokens, TokenKind::Url), "expected Url token for https");
    }

    #[test]
    fn filesystem_path() {
        let tokens = tok("2024-01-15T12:00:00 INFO reading /etc/nginx/nginx.conf");
        assert!(has_kind(&tokens, TokenKind::Path), "expected Path token");
        let path = find_kind(&tokens, TokenKind::Path).unwrap();
        assert!(path.starts_with('/'));
    }

    #[test]
    fn line_without_timestamp_still_parses() {
        // A plain log line without timestamp — should not panic
        let tokens = tok("ERROR no timestamp here");
        assert!(!tokens.is_empty());
        assert!(has_kind(&tokens, TokenKind::Keyword), "ERROR should still be Keyword");
    }

    #[test]
    fn multiple_lines() {
        let input = "2024-01-15T12:00:00 INFO start\n2024-01-15T12:00:01 ERROR fail\n";
        let tokens = tok(input);
        let timestamps: Vec<_> = tokens.iter().filter(|(k, _)| *k == TokenKind::Timestamp).collect();
        assert_eq!(timestamps.len(), 2, "expected two timestamps");
    }
}

pub struct LogGrammar;

impl Grammar for LogGrammar {
    fn tokenize<'a>(&self, input: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();

        for line in input.split_inclusive('\n') {
            tokenize_log_line(&mut tokens, line);
        }

        tokens
    }
}

fn tokenize_log_line<'a>(tokens: &mut Vec<Token<'a>>, line: &'a str) {
    let mut pos = 0;
    let bytes = line.as_bytes();

    // Try to find timestamp at start
    if let Some(ts_end) = find_timestamp_end(line) {
        tokens.push(Token { text: &line[..ts_end], kind: TokenKind::Timestamp });
        pos = ts_end;
    }

    // Scan the rest for log levels, IPs, URLs, paths
    while pos < bytes.len() {
        // Whitespace
        if bytes[pos].is_ascii_whitespace() {
            let start = pos;
            while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }
            tokens.push(Token { text: &line[start..pos], kind: TokenKind::Plain });
            continue;
        }

        // Log levels
        if let Some(level_len) = match_log_level(&line[pos..]) {
            let kind = match &line[pos..pos + level_len].to_ascii_uppercase().as_str() {
                &_ if line[pos..pos + level_len].to_ascii_uppercase().contains("ERR") => TokenKind::Keyword,
                &_ if line[pos..pos + level_len].to_ascii_uppercase().contains("WARN") => TokenKind::Label,
                &_ if line[pos..pos + level_len].to_ascii_uppercase().contains("INFO") => TokenKind::String,
                &_ if line[pos..pos + level_len].to_ascii_uppercase().contains("DEBUG") => TokenKind::Comment,
                &_ if line[pos..pos + level_len].to_ascii_uppercase().contains("TRACE") => TokenKind::Comment,
                _ => TokenKind::Label,
            };
            tokens.push(Token { text: &line[pos..pos + level_len], kind });
            pos += level_len;
            continue;
        }

        // IP addresses (simple v4)
        if bytes[pos].is_ascii_digit() {
            if let Some(ip_len) = match_ipv4(&line[pos..]) {
                tokens.push(Token { text: &line[pos..pos + ip_len], kind: TokenKind::Ip });
                pos += ip_len;
                continue;
            }
        }

        // URLs
        if (line[pos..].starts_with("http://")) || (line[pos..].starts_with("https://")) {
            let start = pos;
            while pos < bytes.len() && !bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }
            tokens.push(Token { text: &line[start..pos], kind: TokenKind::Url });
            continue;
        }

        // Paths: /foo/bar
        if bytes[pos] == b'/' && pos + 1 < bytes.len() && bytes[pos + 1].is_ascii_alphanumeric() {
            let start = pos;
            while pos < bytes.len() && !bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }
            tokens.push(Token { text: &line[start..pos], kind: TokenKind::Path });
            continue;
        }

        // Quoted strings
        if bytes[pos] == b'"' || bytes[pos] == b'\'' {
            let quote = bytes[pos];
            let start = pos;
            pos += 1;
            while pos < bytes.len() && bytes[pos] != quote && bytes[pos] != b'\n' {
                pos += 1;
            }
            if pos < bytes.len() && bytes[pos] == quote {
                pos += 1;
            }
            tokens.push(Token { text: &line[start..pos], kind: TokenKind::String });
            continue;
        }

        // Brackets: [...] common in log formats
        if bytes[pos] == b'[' {
            let start = pos;
            while pos < bytes.len() && bytes[pos] != b']' && bytes[pos] != b'\n' {
                pos += 1;
            }
            if pos < bytes.len() && bytes[pos] == b']' {
                pos += 1;
            }
            tokens.push(Token { text: &line[start..pos], kind: TokenKind::Variable });
            continue;
        }

        // Numbers
        if bytes[pos].is_ascii_digit() {
            let start = pos;
            while pos < bytes.len() && (bytes[pos].is_ascii_digit() || bytes[pos] == b'.') {
                pos += 1;
            }
            tokens.push(Token { text: &line[start..pos], kind: TokenKind::Number });
            continue;
        }

        // Words
        if bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_' {
            let start = pos;
            while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                pos += 1;
            }
            tokens.push(Token { text: &line[start..pos], kind: TokenKind::Plain });
            continue;
        }

        // Everything else (including multi-byte UTF-8)
        let start = pos;
        pos = advance_char(bytes, pos);
        tokens.push(Token { text: &line[start..pos], kind: TokenKind::Plain });
    }
}

fn find_timestamp_end(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    if bytes.len() < 10 {
        return None;
    }

    // ISO: 2024-01-15T12:34:56 or 2024-01-15 12:34:56
    if bytes.get(4) == Some(&b'-') && bytes.get(7) == Some(&b'-') {
        let mut end = 10;
        if end < bytes.len() && (bytes[end] == b'T' || bytes[end] == b' ') {
            end += 1;
            // Time part
            while end < bytes.len() && !bytes[end].is_ascii_whitespace() && bytes[end] != b']' {
                end += 1;
            }
        }
        return Some(end);
    }

    // Syslog: "Jan 15 12:34:56"
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    if months.iter().any(|m| line.starts_with(m)) && bytes.len() > 15 {
        // "Mon DD HH:MM:SS"
        return Some(15);
    }

    None
}

fn match_log_level(s: &str) -> Option<usize> {
    let levels = [
        "ERROR", "WARN", "WARNING", "INFO", "DEBUG", "TRACE", "FATAL", "CRITICAL",
        "error", "warn", "warning", "info", "debug", "trace", "fatal", "critical",
        "ERR", "WRN", "INF", "DBG", "TRC",
    ];
    for level in &levels {
        if s.starts_with(level) {
            // Must be followed by non-alphanumeric
            let len = level.len();
            if len >= s.len() || !s.as_bytes()[len].is_ascii_alphanumeric() {
                return Some(len);
            }
        }
    }
    None
}

fn match_ipv4(s: &str) -> Option<usize> {
    let mut pos = 0;
    let bytes = s.as_bytes();
    let mut octets = 0;

    for _ in 0..4 {
        if pos >= bytes.len() || !bytes[pos].is_ascii_digit() {
            return None;
        }
        let start = pos;
        while pos < bytes.len() && bytes[pos].is_ascii_digit() {
            pos += 1;
        }
        if pos - start > 3 {
            return None;
        }
        octets += 1;
        if octets < 4 {
            if pos >= bytes.len() || bytes[pos] != b'.' {
                return None;
            }
            pos += 1;
        }
    }

    // Must not be followed by a digit or dot (would be a version number etc.)
    if pos < bytes.len() && (bytes[pos].is_ascii_digit() || bytes[pos] == b'.') {
        return None;
    }

    Some(pos)
}
