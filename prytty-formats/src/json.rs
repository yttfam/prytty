/// Maximum nesting depth for JSON formatting. Prevents quadratic output
/// from pathologically nested input like `[[[[[...`.
const MAX_DEPTH: usize = 256;

/// Pretty-print JSON with indentation.
/// This is a lightweight formatter — no serde, no parsing into a tree.
/// Just re-indents valid-ish JSON.
pub fn format_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2);
    let mut indent = 0usize;
    let mut in_string = false;
    let mut chars = input.char_indices().peekable();

    while let Some((i, ch)) = chars.next() {
        if in_string {
            out.push(ch);
            if ch == '\\' {
                // Copy escaped character as-is (preserves multi-byte UTF-8)
                if let Some(&(_, next_ch)) = chars.peek() {
                    out.push(next_ch);
                    chars.next();
                }
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
                out.push('"');
            }
            '{' | '[' => {
                out.push(ch);
                // Check if empty
                let next_ch = peek_non_ws_char(&mut chars);
                if next_ch == Some('}') || next_ch == Some(']') {
                    // Empty object/array — don't add newline
                } else if indent < MAX_DEPTH {
                    indent += 1;
                    out.push('\n');
                    push_indent(&mut out, indent);
                } else {
                    // Beyond max depth — stop indenting to prevent quadratic output
                    indent += 1;
                }
            }
            '}' | ']' => {
                // Check if previous meaningful char was { or [
                let prev = prev_non_ws(&out);
                if prev == Some(b'{') || prev == Some(b'[') {
                    // Empty — no indent change
                } else if indent <= MAX_DEPTH {
                    indent = indent.saturating_sub(1);
                    out.push('\n');
                    push_indent(&mut out, indent);
                } else {
                    indent = indent.saturating_sub(1);
                }
                out.push(ch);
            }
            ',' => {
                out.push(',');
                if indent <= MAX_DEPTH {
                    out.push('\n');
                    push_indent(&mut out, indent);
                }
            }
            ':' => {
                out.push(':');
                out.push(' ');
            }
            ' ' | '\t' | '\n' | '\r' => {
                // Skip original whitespace
            }
            _ => {
                out.push(ch);
            }
        }
        // Note: no manual `i += 1` needed — char_indices iterator handles advancement
        let _ = i; // suppress unused warning
    }

    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_object() {
        let out = format_json("{}");
        // Empty object stays on one line
        assert!(out.contains("{}"), "empty object should not be expanded");
    }

    #[test]
    fn empty_array() {
        let out = format_json("[]");
        assert!(out.contains("[]"), "empty array should not be expanded");
    }

    #[test]
    fn simple_object_indented() {
        let out = format_json(r#"{"a":1}"#);
        assert!(out.contains("\"a\": 1"), "key-value should have space after colon");
        assert!(out.contains('\n'), "output should have newlines");
    }

    #[test]
    fn colon_gets_space() {
        let out = format_json(r#"{"key":"value"}"#);
        assert!(out.contains("\"key\": \"value\""), "colon should be followed by space");
    }

    #[test]
    fn comma_triggers_newline() {
        let out = format_json(r#"{"a":1,"b":2}"#);
        // After a comma there should be a newline
        assert!(out.contains(",\n"), "comma should be followed by newline");
    }

    #[test]
    fn nested_object_indentation() {
        let out = format_json(r#"{"outer":{"inner":42}}"#);
        // "inner" should appear at deeper indentation than "outer"
        let outer_line = out.lines().find(|l| l.contains("\"outer\"")).expect("outer key not found");
        let inner_line = out.lines().find(|l| l.contains("\"inner\"")).expect("inner key not found");
        let outer_indent = outer_line.len() - outer_line.trim_start().len();
        let inner_indent = inner_line.len() - inner_line.trim_start().len();
        assert!(inner_indent > outer_indent, "inner should be indented more than outer");
    }

    #[test]
    fn array_items_indented() {
        let out = format_json("[1,2,3]");
        assert!(out.contains("1"), "1 should be in output");
        assert!(out.contains("2"), "2 should be in output");
        assert!(out.contains("3"), "3 should be in output");
        // Items separated by commas + newlines
        assert!(out.contains(",\n"), "comma should be followed by newline in array");
    }

    #[test]
    fn string_with_escape_preserved() {
        let out = format_json(r#"{"msg":"hello\"world"}"#);
        assert!(out.contains("hello\\\"world") || out.contains("hello\"world"),
            "escape sequence in string should be preserved");
    }

    #[test]
    fn string_with_colon_not_split() {
        // Colon inside a string value should not get extra space inserted
        let out = format_json(r#"{"url":"http://example.com"}"#);
        assert!(out.contains("http://example.com"), "URL inside string should not be modified");
    }

    #[test]
    fn output_ends_with_newline() {
        let out = format_json("{}");
        assert!(out.ends_with('\n'), "output should end with newline");
    }

    #[test]
    fn compact_input_reformatted() {
        let compact = r#"{"name":"alice","age":30}"#;
        let out = format_json(compact);
        // Output should be longer than input (added whitespace)
        assert!(out.len() > compact.len(), "formatted output should be longer than compact input");
    }

    #[test]
    fn already_formatted_input_is_equivalent() {
        // Feeding already-formatted JSON strips original whitespace and re-formats
        let input = "{\n  \"a\": 1\n}\n";
        let out = format_json(input);
        assert!(out.contains("\"a\": 1"), "key-value should survive re-formatting");
    }

    #[test]
    fn indentation_is_two_spaces() {
        let out = format_json(r#"{"x":1}"#);
        // The "x" line should be indented by exactly 2 spaces (one level)
        let line = out.lines().find(|l| l.contains("\"x\"")).expect("x key not found");
        assert!(line.starts_with("  "), "one-level indent should be 2 spaces");
        assert!(!line.starts_with("   "), "one-level indent should not be more than 2 spaces");
    }

    #[test]
    fn array_of_objects() {
        let out = format_json(r#"[{"a":1},{"b":2}]"#);
        assert!(out.contains("\"a\": 1"));
        assert!(out.contains("\"b\": 2"));
    }
}

fn peek_non_ws_char(
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
) -> Option<char> {
    // Clone the iterator to peek ahead without consuming
    let mut clone = chars.clone();
    loop {
        match clone.next() {
            Some((_, c)) if c.is_ascii_whitespace() => continue,
            Some((_, c)) => return Some(c),
            None => return None,
        }
    }
}

fn prev_non_ws(s: &str) -> Option<u8> {
    s.as_bytes().iter().rev().find(|b| !b.is_ascii_whitespace()).copied()
}

fn push_indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("  ");
    }
}
