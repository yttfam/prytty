use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn prytty() -> Command {
    Command::cargo_bin("prytty").expect("prytty binary not found")
}

// ── 1. Stdin pipe mode ───────────────────────────────────────────────────────
// Echo Rust code via stdin with --color true; output must contain ANSI escapes.
#[test]
fn stdin_pipe_produces_ansi() {
    prytty()
        .args(["--color", "true", "--language", "rust"])
        .write_stdin("fn main() { let x = 42; }\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b["));
}

// ── 2. File mode ─────────────────────────────────────────────────────────────
// Write a temp .rs file, run prytty on it, verify colored output.
#[test]
fn file_mode_produces_ansi() {
    let mut tmp = NamedTempFile::with_suffix(".rs").expect("tempfile");
    writeln!(tmp, "fn main() {{\n    let x: u32 = 1;\n}}").unwrap();

    prytty()
        .args(["--color", "true", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b["));
}

// ── 3. Language flag ─────────────────────────────────────────────────────────
// Plain text forced to --language rust should be highlighted as Rust.
#[test]
fn language_flag_overrides_auto_detection() {
    // "fn" is a Rust keyword; with --language rust it gets a color escape.
    prytty()
        .args(["--color", "true", "--language", "rust"])
        .write_stdin("fn greet() {}\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b["));
}

// ── 4. JSON format mode ──────────────────────────────────────────────────────
// Minified JSON with --format should be indented and colored.
#[test]
fn json_format_mode_indents_and_colors() {
    prytty()
        .args(["--color", "true", "--format", "--language", "json"])
        .write_stdin(r#"{"name":"prytty","version":"0.1.0"}"#)
        .assert()
        .success()
        // Formatted JSON must contain a newline (indentation introduced)
        .stdout(predicate::str::contains("\n"))
        // And ANSI coloring
        .stdout(predicate::str::contains("\x1b["));
}

// ── 5. Auto-detection: JSON content without a language flag ──────────────────
// Content starting with `{` and containing `":` is auto-detected as JSON.
// JSON keys become TokenKind::Key which are colored.
#[test]
fn json_auto_detection_highlights_keys() {
    prytty()
        .args(["--color", "true"])
        .write_stdin("{\"status\": \"ok\", \"code\": 200}\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b["));
}

// ── 6. No color when --color none ────────────────────────────────────────────
// With --color none, no foreground color escape sequences should appear.
// The renderer may still emit a bare reset \x1b[0m at the end, but no
// \x1b[38;... (truecolor), \x1b[38;5;... (256), or \x1b[3Xm (16-color fg)
// sequences should be present.
#[test]
fn no_ansi_when_color_none() {
    prytty()
        .args(["--color", "none", "--language", "rust"])
        .write_stdin("fn main() { let x = 1; }\n")
        .assert()
        .success()
        // No foreground color assignments
        .stdout(predicate::str::contains("\x1b[38;").not())
        // No bold/italic styling
        .stdout(predicate::str::contains("\x1b[1m").not())
        .stdout(predicate::str::contains("\x1b[3m").not());
}

// ── 7. File not found ────────────────────────────────────────────────────────
// prytty must exit non-zero and print an error to stderr.
#[test]
fn file_not_found_exits_nonzero_with_stderr() {
    prytty()
        .arg("/tmp/this_file_does_not_exist_prytty_e2e_test.rs")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ── 8. Empty input ───────────────────────────────────────────────────────────
// Empty stdin must not crash; exit code 0.
#[test]
fn empty_input_clean_exit() {
    prytty()
        .args(["--color", "true"])
        .write_stdin("")
        .assert()
        .success();
}

// ── 9. Help flag ─────────────────────────────────────────────────────────────
// --help must exit 0 and mention usage.
#[test]
fn help_flag_shows_usage() {
    prytty()
        .arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("prytty")
                .and(predicate::str::contains("Usage").or(predicate::str::contains("usage"))),
        );
}

// ── 10. YAML auto-detection ──────────────────────────────────────────────────
// YAML content (key: value lines) is auto-detected and produces ANSI output.
#[test]
fn yaml_auto_detection_produces_ansi() {
    // "---\n" triggers YAML detection via the `starts_with("---")` heuristic.
    // The "name:" line emits a TokenKind::Key token which is colored.
    prytty()
        .args(["--color", "true"])
        .write_stdin("---\nname: prytty\nversion: 0.1.0\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b["));
}

// ── 11. Diff auto-detection ──────────────────────────────────────────────────
// A diff with +/- lines should be auto-detected and colored.
#[test]
fn diff_auto_detection_produces_ansi() {
    let diff = "--- a/main.rs\n+++ b/main.rs\n@@ -1,3 +1,3 @@\n-let x = 1;\n+let x = 2;\n";
    prytty()
        .args(["--color", "true"])
        .write_stdin(diff)
        .assert()
        .success()
        // "--- " header line is a Label token → gets colored
        .stdout(predicate::str::contains("\x1b["));
}
