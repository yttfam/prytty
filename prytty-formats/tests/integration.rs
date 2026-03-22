use prytty_formats::format_json;

// ── format_json: basic behaviour ─────────────────────────────────────────────

#[test]
fn format_json_empty_object() {
    let out = format_json("{}");
    // Empty objects are kept on one line without extra indentation
    assert!(out.contains("{}"));
}

#[test]
fn format_json_empty_array() {
    let out = format_json("[]");
    assert!(out.contains("[]"));
}

#[test]
fn format_json_ends_with_newline() {
    let out = format_json(r#"{"a":1}"#);
    assert!(out.ends_with('\n'), "output should end with newline: {:?}", out);
}

// ── Minified → readable ───────────────────────────────────────────────────────

#[test]
fn format_json_minified_object_is_expanded() {
    let minified = r#"{"name":"prytty","version":"0.1.0"}"#;
    let formatted = format_json(minified);
    // The formatted output should contain newlines (it was expanded)
    let newline_count = formatted.chars().filter(|&c| c == '\n').count();
    assert!(newline_count > 1, "expected multiple newlines in formatted output: {:?}", formatted);
    // Keys and values must still be present
    assert!(formatted.contains("name"));
    assert!(formatted.contains("prytty"));
    assert!(formatted.contains("version"));
    assert!(formatted.contains("0.1.0"));
}

#[test]
fn format_json_minified_adds_colon_space() {
    let minified = r#"{"key":"value"}"#;
    let formatted = format_json(minified);
    // Formatter adds space after colon
    assert!(formatted.contains(": "), "expected ': ' in formatted output: {:?}", formatted);
}

#[test]
fn format_json_minified_array_is_expanded() {
    let minified = r#"[1,2,3]"#;
    let formatted = format_json(minified);
    let newline_count = formatted.chars().filter(|&c| c == '\n').count();
    assert!(newline_count > 1, "expected multiple newlines in expanded array: {:?}", formatted);
    assert!(formatted.contains('1'));
    assert!(formatted.contains('2'));
    assert!(formatted.contains('3'));
}

// ── Round-trip: already-formatted JSON ───────────────────────────────────────

#[test]
fn format_json_round_trip_simple() {
    // A simple already-formatted JSON
    let original = "{\n  \"name\": \"prytty\"\n}\n";
    let first = format_json(original);
    let second = format_json(&first);
    assert_eq!(first, second, "format_json should be idempotent:\nfirst:\n{first}\nsecond:\n{second}");
}

#[test]
fn format_json_round_trip_array() {
    let original = "[\n  1,\n  2,\n  3\n]\n";
    let first = format_json(original);
    let second = format_json(&first);
    assert_eq!(first, second, "format_json should be idempotent for arrays");
}

#[test]
fn format_json_round_trip_multiple_keys() {
    let original = "{\n  \"a\": 1,\n  \"b\": \"hello\",\n  \"c\": true\n}\n";
    let first = format_json(original);
    let second = format_json(&first);
    assert_eq!(first, second, "format_json should be idempotent");
}

// ── Nested structures ─────────────────────────────────────────────────────────

#[test]
fn format_json_nested_object() {
    let input = r#"{"outer":{"inner":"value"}}"#;
    let formatted = format_json(input);
    assert!(formatted.contains("outer"));
    assert!(formatted.contains("inner"));
    assert!(formatted.contains("value"));
    // Should have multiple indentation levels (at least 4 spaces somewhere)
    assert!(formatted.contains("    "), "expected 4-space indent for nested object: {:?}", formatted);
    assert!(formatted.ends_with('\n'));
}

#[test]
fn format_json_nested_array_in_object() {
    let input = r#"{"items":[1,2,3]}"#;
    let formatted = format_json(input);
    assert!(formatted.contains("items"));
    assert!(formatted.contains('1'));
    assert!(formatted.ends_with('\n'));
}

#[test]
fn format_json_deeply_nested() {
    let input = r#"{"a":{"b":{"c":42}}}"#;
    let formatted = format_json(input);
    assert!(formatted.contains("a"));
    assert!(formatted.contains("b"));
    assert!(formatted.contains("c"));
    assert!(formatted.contains("42"));
    // Should be indented multiple levels
    assert!(formatted.contains("      "), "expected 6-space indent for 3 levels deep: {:?}", formatted);
    assert!(formatted.ends_with('\n'));
}

#[test]
fn format_json_array_of_objects() {
    let input = r#"[{"name":"alice"},{"name":"bob"}]"#;
    let formatted = format_json(input);
    assert!(formatted.contains("alice"));
    assert!(formatted.contains("bob"));
    assert!(formatted.ends_with('\n'));
}

// ── Content preservation ──────────────────────────────────────────────────────

#[test]
fn format_json_preserves_string_values() {
    let input = r#"{"msg":"hello world","path":"/usr/bin"}"#;
    let formatted = format_json(input);
    assert!(formatted.contains("hello world"));
    assert!(formatted.contains("/usr/bin"));
}

#[test]
fn format_json_preserves_numeric_values() {
    let input = r#"{"int":42,"float":3.14,"neg":-1}"#;
    let formatted = format_json(input);
    assert!(formatted.contains("42"));
    assert!(formatted.contains("3.14"));
    assert!(formatted.contains("-1"));
}

#[test]
fn format_json_preserves_boolean_and_null() {
    let input = r#"{"a":true,"b":false,"c":null}"#;
    let formatted = format_json(input);
    assert!(formatted.contains("true"));
    assert!(formatted.contains("false"));
    assert!(formatted.contains("null"));
}

#[test]
fn format_json_preserves_escaped_strings() {
    let input = r#"{"msg":"say \"hello\""}"#;
    let formatted = format_json(input);
    assert!(formatted.contains("say \\\"hello\\\""), "escaped quotes must be preserved: {:?}", formatted);
}
