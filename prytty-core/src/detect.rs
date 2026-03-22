use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    Json,
    Yaml,
    Toml,
    Diff,
    Log,
    Generic,
}

impl Language {
    pub fn name(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Diff => "diff",
            Self::Log => "log",
            Self::Generic => "generic",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "rust" | "rs" => Some(Self::Rust),
            "python" | "py" => Some(Self::Python),
            "json" => Some(Self::Json),
            "yaml" | "yml" => Some(Self::Yaml),
            "toml" => Some(Self::Toml),
            "diff" | "patch" => Some(Self::Diff),
            "log" | "syslog" => Some(Self::Log),
            _ => None,
        }
    }
}

/// Detect language from a file path (extension-based).
pub fn detect_language(path: Option<&Path>, content: &str) -> Language {
    // 1. Try file extension
    if let Some(path) = path {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext {
                "rs" => return Language::Rust,
                "py" | "pyw" => return Language::Python,
                "json" => return Language::Json,
                "yaml" | "yml" => return Language::Yaml,
                "toml" => return Language::Toml,
                "diff" | "patch" => return Language::Diff,
                "log" => return Language::Log,
                _ => {}
            }
        }
    }

    // 2. Try shebang
    if let Some(first_line) = content.lines().next() {
        if first_line.starts_with("#!") {
            if first_line.contains("python") {
                return Language::Python;
            }
            if first_line.contains("rustc") {
                return Language::Rust;
            }
        }
    }

    // 3. Content heuristics
    detect_from_content(content)
}

fn detect_from_content(content: &str) -> Language {
    let trimmed = content.trim_start();

    // JSON: starts with { or [
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        // Quick validation: does it look like JSON?
        if trimmed.contains("\":") || trimmed.contains("\": ") {
            return Language::Json;
        }
        // Could still be JSON array of primitives
        if trimmed.starts_with('[') {
            return Language::Json;
        }
    }

    // Diff: starts with diff/--- /+++ /@@
    if trimmed.starts_with("diff ") || trimmed.starts_with("--- ") || trimmed.starts_with("+++ ") {
        return Language::Diff;
    }

    // YAML: starts with --- or key: value pattern
    if trimmed.starts_with("---") && !trimmed.starts_with("--- a/") {
        return Language::Yaml;
    }

    // TOML: starts with [section] or key = value
    if trimmed.starts_with('[') && trimmed.contains(']') && !trimmed.contains("\":")  {
        return Language::Toml;
    }

    // Rust: fn, let, use, mod, pub, impl, struct, enum
    let rust_indicators = ["fn ", "let ", "use ", "mod ", "pub ", "impl ", "struct ", "enum "];
    let rust_score: usize = rust_indicators.iter().filter(|kw| content.contains(*kw)).count();
    if rust_score >= 2 {
        return Language::Rust;
    }

    // Python: def, import, from...import, class, if __name__
    let py_indicators = ["def ", "import ", "from ", "class ", "if __name__", "print("];
    let py_score: usize = py_indicators.iter().filter(|kw| content.contains(*kw)).count();
    if py_score >= 2 {
        return Language::Python;
    }

    // Log: timestamp patterns at line start
    let first_lines: Vec<&str> = content.lines().take(5).collect();
    let log_like = first_lines.iter().filter(|l| looks_like_log_line(l)).count();
    if log_like >= 2 {
        return Language::Log;
    }

    Language::Generic
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // --- Language::name / from_name ---

    #[test]
    fn language_names() {
        assert_eq!(Language::Rust.name(), "rust");
        assert_eq!(Language::Python.name(), "python");
        assert_eq!(Language::Json.name(), "json");
        assert_eq!(Language::Yaml.name(), "yaml");
        assert_eq!(Language::Toml.name(), "toml");
        assert_eq!(Language::Diff.name(), "diff");
        assert_eq!(Language::Log.name(), "log");
        assert_eq!(Language::Generic.name(), "generic");
    }

    #[test]
    fn from_name_roundtrip() {
        assert_eq!(Language::from_name("rust"), Some(Language::Rust));
        assert_eq!(Language::from_name("rs"), Some(Language::Rust));
        assert_eq!(Language::from_name("python"), Some(Language::Python));
        assert_eq!(Language::from_name("py"), Some(Language::Python));
        assert_eq!(Language::from_name("json"), Some(Language::Json));
        assert_eq!(Language::from_name("yaml"), Some(Language::Yaml));
        assert_eq!(Language::from_name("yml"), Some(Language::Yaml));
        assert_eq!(Language::from_name("toml"), Some(Language::Toml));
        assert_eq!(Language::from_name("diff"), Some(Language::Diff));
        assert_eq!(Language::from_name("patch"), Some(Language::Diff));
        assert_eq!(Language::from_name("log"), Some(Language::Log));
        assert_eq!(Language::from_name("syslog"), Some(Language::Log));
        assert_eq!(Language::from_name("unknown"), None);
    }

    #[test]
    fn from_name_case_insensitive() {
        assert_eq!(Language::from_name("RUST"), Some(Language::Rust));
        assert_eq!(Language::from_name("JSON"), Some(Language::Json));
    }

    // --- Extension detection ---

    #[test]
    fn detect_by_extension_rs() {
        assert_eq!(detect_language(Some(Path::new("foo.rs")), ""), Language::Rust);
    }

    #[test]
    fn detect_by_extension_py() {
        assert_eq!(detect_language(Some(Path::new("foo.py")), ""), Language::Python);
        assert_eq!(detect_language(Some(Path::new("foo.pyw")), ""), Language::Python);
    }

    #[test]
    fn detect_by_extension_json() {
        assert_eq!(detect_language(Some(Path::new("data.json")), ""), Language::Json);
    }

    #[test]
    fn detect_by_extension_yaml() {
        assert_eq!(detect_language(Some(Path::new("config.yaml")), ""), Language::Yaml);
        assert_eq!(detect_language(Some(Path::new("config.yml")), ""), Language::Yaml);
    }

    #[test]
    fn detect_by_extension_toml() {
        assert_eq!(detect_language(Some(Path::new("Cargo.toml")), ""), Language::Toml);
    }

    #[test]
    fn detect_by_extension_diff() {
        assert_eq!(detect_language(Some(Path::new("changes.diff")), ""), Language::Diff);
        assert_eq!(detect_language(Some(Path::new("changes.patch")), ""), Language::Diff);
    }

    #[test]
    fn detect_by_extension_log() {
        assert_eq!(detect_language(Some(Path::new("app.log")), ""), Language::Log);
    }

    // --- Shebang detection ---

    #[test]
    fn detect_python_shebang() {
        let content = "#!/usr/bin/env python3\nprint('hi')";
        assert_eq!(detect_language(None, content), Language::Python);
    }

    #[test]
    fn detect_python_shebang_path() {
        let content = "#!/usr/bin/python\nx = 1";
        assert_eq!(detect_language(None, content), Language::Python);
    }

    // --- Content heuristics ---

    #[test]
    fn detect_json_object_from_content() {
        let content = r#"{"key": "value"}"#;
        assert_eq!(detect_language(None, content), Language::Json);
    }

    #[test]
    fn detect_json_array_from_content() {
        let content = "[1, 2, 3]";
        assert_eq!(detect_language(None, content), Language::Json);
    }

    #[test]
    fn detect_diff_from_content() {
        let content = "diff --git a/foo.rs b/foo.rs\n--- a/foo.rs\n+++ b/foo.rs";
        assert_eq!(detect_language(None, content), Language::Diff);
    }

    #[test]
    fn detect_yaml_from_document_marker() {
        let content = "---\nfoo: bar\nbaz: 42";
        assert_eq!(detect_language(None, content), Language::Yaml);
    }

    #[test]
    fn detect_rust_from_content() {
        let content = "fn main() {\n    let x = 42;\n    println!(\"{}\", x);\n}";
        assert_eq!(detect_language(None, content), Language::Rust);
    }

    #[test]
    fn detect_python_from_content() {
        let content = "def foo():\n    import os\n    from sys import path\n    return path";
        assert_eq!(detect_language(None, content), Language::Python);
    }

    #[test]
    fn detect_log_from_iso_timestamps() {
        let content = "2024-01-15T12:34:56 INFO starting\n2024-01-15T12:34:57 ERROR crash";
        assert_eq!(detect_language(None, content), Language::Log);
    }

    #[test]
    fn detect_log_from_syslog_timestamps() {
        let content = "Jan 15 12:34:56 host sshd: session opened\nJan 15 12:34:57 host sshd: session closed";
        assert_eq!(detect_language(None, content), Language::Log);
    }

    // --- Edge cases ---

    #[test]
    fn detect_empty_input_is_generic() {
        assert_eq!(detect_language(None, ""), Language::Generic);
    }

    #[test]
    fn detect_whitespace_only_is_generic() {
        assert_eq!(detect_language(None, "   \n\n\t  "), Language::Generic);
    }

    #[test]
    fn detect_no_path_plain_text_is_generic() {
        assert_eq!(detect_language(None, "hello world"), Language::Generic);
    }

    #[test]
    fn extension_takes_precedence_over_content() {
        // Content looks like JSON but extension says Rust
        let content = r#"{"key": "value"}"#;
        assert_eq!(detect_language(Some(Path::new("foo.rs")), content), Language::Rust);
    }
}

fn looks_like_log_line(line: &str) -> bool {
    let line = line.trim_start();
    // ISO timestamp: 2024-01-15T... or 2024-01-15 ...
    if line.len() > 10 && line.as_bytes().get(4) == Some(&b'-') && line.as_bytes().get(7) == Some(&b'-') {
        return true;
    }
    // Syslog-style: "Jan 15 ..."
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    if months.iter().any(|m| line.starts_with(m)) {
        return true;
    }
    false
}
