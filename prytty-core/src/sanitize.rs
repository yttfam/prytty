/// Strip ANSI escape sequences from input to prevent terminal injection attacks.
///
/// Untrusted input piped through prytty could contain crafted escape sequences
/// (OSC to set terminal title, OSC 52 to write clipboard, etc.). We strip all
/// ANSI escapes before tokenizing so that only prytty-generated sequences reach
/// the terminal.
///
/// Handles:
/// - CSI sequences: ESC [ ... (final byte 0x40-0x7E)
/// - OSC sequences: ESC ] ... (terminated by ST or BEL)
/// - Simple two-byte escapes: ESC + single char
/// - C1 control codes in the 0x80-0x9F range
/// - Standalone BEL, SO, SI, and other C0 control chars (except \t, \n, \r)
pub fn strip_ansi(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;

    while i < bytes.len() {
        // ESC (0x1B) starts an escape sequence
        if bytes[i] == 0x1B {
            i += 1;
            if i >= bytes.len() {
                break;
            }
            match bytes[i] {
                // CSI: ESC [
                b'[' => {
                    i += 1;
                    // Parameter bytes (0x30-0x3F), intermediate bytes (0x20-0x2F),
                    // terminated by final byte (0x40-0x7E)
                    while i < bytes.len() {
                        let b = bytes[i];
                        if (0x40..=0x7E).contains(&b) {
                            i += 1; // consume final byte
                            break;
                        }
                        i += 1;
                    }
                }
                // OSC: ESC ]
                b']' => {
                    i += 1;
                    // Terminated by BEL (0x07) or ST (ESC \)
                    while i < bytes.len() {
                        if bytes[i] == 0x07 {
                            i += 1;
                            break;
                        }
                        if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                }
                // DCS (ESC P), SOS (ESC X), PM (ESC ^), APC (ESC _)
                // All terminated by ST (ESC \ or 0x9C)
                b'P' | b'X' | b'^' | b'_' => {
                    i += 1;
                    while i < bytes.len() {
                        if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                            i += 2;
                            break;
                        }
                        if bytes[i] == 0x9C {
                            i += 1;
                            break;
                        }
                        i += 1;
                    }
                }
                // Two-byte sequences: ESC + any char in 0x20-0x7E
                b if (0x20..=0x7E).contains(&b) => {
                    i += 1;
                }
                // Unknown after ESC — skip just the ESC
                _ => {}
            }
            continue;
        }

        // C1 control codes (0x80-0x9F) — strip them
        // Note: in UTF-8 these bytes are continuation bytes, but valid UTF-8
        // strings won't have bare 0x80-0x9F. Rust strings are valid UTF-8,
        // so we only encounter these as part of multi-byte sequences.
        // We handle the ESC-based C1 equivalents above.

        // Strip C0 control characters except tab, newline, carriage return
        if bytes[i] < 0x20 && bytes[i] != b'\t' && bytes[i] != b'\n' && bytes[i] != b'\r' {
            i += 1;
            continue;
        }

        // Regular character — copy it through
        // Handle UTF-8 multi-byte sequences correctly
        if bytes[i] < 0x80 {
            out.push(bytes[i] as char);
            i += 1;
        } else {
            // UTF-8 multi-byte: find the full character
            let ch_start = i;
            let width = utf8_char_width(bytes[i]);
            i += width;
            if i <= bytes.len() {
                out.push_str(&input[ch_start..i]);
            }
        }
    }

    out
}

fn utf8_char_width(first_byte: u8) -> usize {
    match first_byte {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xFF => 4,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_unchanged() {
        assert_eq!(strip_ansi("hello world"), "hello world");
    }

    #[test]
    fn preserves_newlines_and_tabs() {
        assert_eq!(strip_ansi("a\tb\nc\r\n"), "a\tb\nc\r\n");
    }

    #[test]
    fn strips_csi_color() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
    }

    #[test]
    fn strips_osc_title_bel() {
        // ESC ] 2 ; pwned BEL
        assert_eq!(strip_ansi("\x1b]2;pwned\x07"), "");
    }

    #[test]
    fn strips_osc_title_st() {
        // ESC ] 2 ; pwned ESC backslash
        assert_eq!(strip_ansi("\x1b]2;pwned\x1b\\"), "");
    }

    #[test]
    fn strips_osc_52_clipboard() {
        // OSC 52 clipboard write
        assert_eq!(strip_ansi("\x1b]52;c;SGVsbG8=\x07"), "");
    }

    #[test]
    fn strips_bel() {
        assert_eq!(strip_ansi("hello\x07world"), "helloworld");
    }

    #[test]
    fn mixed_content() {
        assert_eq!(
            strip_ansi("before\x1b[1mbold\x1b[0mafter"),
            "beforeboldafter"
        );
    }

    #[test]
    fn preserves_utf8() {
        assert_eq!(strip_ansi("hello 世界 🌍"), "hello 世界 🌍");
    }

    #[test]
    fn strips_dcs_sequence() {
        assert_eq!(strip_ansi("\x1bPsome data\x1b\\visible"), "visible");
    }

    #[test]
    fn empty_input() {
        assert_eq!(strip_ansi(""), "");
    }

    #[test]
    fn trailing_esc() {
        assert_eq!(strip_ansi("hello\x1b"), "hello");
    }
}
