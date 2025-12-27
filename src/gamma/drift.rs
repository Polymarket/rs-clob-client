//! API drift detection - logs unknown fields in API responses.
//!
//! This module detects when the Gamma API returns fields that aren't defined
//! in our response types. This helps identify API schema changes early.
//!
//! Drift detection is only active when the `tracing` feature is enabled.

use serde::Serialize;
use serde_json::Value;

/// Detects and logs unknown fields by comparing original JSON against a round-tripped value.
///
/// The approach: deserialize JSON to a typed struct T, then serialize T back to JSON.
/// Fields present in the original JSON but missing from the round-tripped JSON are unknown.
///
/// We skip null values to avoid false positives from `#[serde(skip_serializing_if)]`.
pub fn detect_and_log<T: Serialize + ?Sized>(original: &Value, typed: &T, path: &str) {
    let Ok(round_tripped) = serde_json::to_value(typed) else {
        return; // Don't block on serialization failures
    };

    let unknown = find_unknown_fields(original, &round_tripped, "");

    for (field_path, value) in unknown {
        tracing::warn!(
            endpoint = %path,
            field = %field_path,
            value = %truncate_value(&value),
            "API drift: unknown field in response"
        );
    }
}

/// Recursively finds fields in `original` that are missing from `round_tripped`.
fn find_unknown_fields(
    original: &Value,
    round_tripped: &Value,
    prefix: &str,
) -> Vec<(String, Value)> {
    let mut result = Vec::new();

    if let (Value::Object(orig), Value::Object(rt)) = (original, round_tripped) {
        for (key, value) in orig {
            let field_path = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{prefix}.{key}")
            };

            if !rt.contains_key(key) {
                // Field in original but not in round-trip = unknown
                // Skip nulls to avoid false positives from skip_serializing_if
                if !value.is_null() {
                    result.push((field_path, value.clone()));
                }
            } else if let (Value::Object(_), Value::Object(_)) = (value, &rt[key]) {
                // Recurse into nested objects
                result.extend(find_unknown_fields(value, &rt[key], &field_path));
            }
            // Note: We don't recurse into arrays to avoid complexity and noise.
            // Array element drift would require sampling which adds complexity.
        }
    }

    result
}

/// Truncates large values to avoid log spam.
/// Uses UTF-8 boundary-aware slicing to avoid panics on multi-byte characters.
#[expect(
    clippy::string_slice,
    reason = "safe: we check is_char_boundary(end) before slicing"
)]
fn truncate_value(value: &Value) -> String {
    let s = value.to_string();
    if s.len() > 200 {
        // Find the last valid UTF-8 boundary at or before index 200
        let mut end = 200.min(s.len());
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        if end == 0 {
            // Extremely unlikely: no valid boundary found before index 200.
            // Fall back to an ellipsis without slicing.
            "...".to_owned()
        } else {
            format!("{}...", &s[..end])
        }
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestStruct {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    }

    #[test]
    fn finds_unknown_field() {
        let original = json!({
            "id": "123",
            "name": "test",
            "unknown_field": "surprise!"
        });

        let typed = TestStruct {
            id: "123".to_owned(),
            name: Some("test".to_owned()),
        };

        let round_tripped = serde_json::to_value(&typed).unwrap();
        let unknown = find_unknown_fields(&original, &round_tripped, "");

        assert_eq!(unknown.len(), 1);
        assert_eq!(unknown[0].0, "unknown_field");
        assert_eq!(unknown[0].1, json!("surprise!"));
    }

    #[test]
    fn ignores_null_values() {
        // When original has null and round-trip skips it, don't report as unknown
        let original = json!({
            "id": "123",
            "name": null
        });

        let typed = TestStruct {
            id: "123".to_owned(),
            name: None,
        };

        let round_tripped = serde_json::to_value(&typed).unwrap();
        let unknown = find_unknown_fields(&original, &round_tripped, "");

        assert!(unknown.is_empty());
    }

    #[test]
    fn finds_nested_unknown_field() {
        #[derive(Debug, Serialize, Deserialize)]
        struct Outer {
            inner: Inner,
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct Inner {
            value: i32,
        }

        let original = json!({
            "inner": {
                "value": 42,
                "secret": "hidden"
            }
        });

        let typed = Outer {
            inner: Inner { value: 42 },
        };

        let round_tripped = serde_json::to_value(&typed).unwrap();
        let unknown = find_unknown_fields(&original, &round_tripped, "");

        assert_eq!(unknown.len(), 1);
        assert_eq!(unknown[0].0, "inner.secret");
    }

    #[test]
    fn no_unknown_fields() {
        let original = json!({
            "id": "123",
            "name": "test"
        });

        let typed = TestStruct {
            id: "123".to_owned(),
            name: Some("test".to_owned()),
        };

        let round_tripped = serde_json::to_value(&typed).unwrap();
        let unknown = find_unknown_fields(&original, &round_tripped, "");

        assert!(unknown.is_empty());
    }

    #[test]
    fn truncate_value_handles_utf8() {
        // Create a string with multi-byte UTF-8 characters that would cause
        // a panic if we naively slice at byte 200
        let long_emoji_string = "🎉".repeat(60); // Each emoji is 4 bytes = 240 bytes
        let value = Value::String(long_emoji_string);

        // This should not panic
        let truncated = truncate_value(&value);

        // Should end with "..." and not exceed ~200 chars
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() <= 210); // Some buffer for the "..." suffix
    }

    #[test]
    fn truncate_value_short_string() {
        let value = Value::String("short".to_owned());
        let result = truncate_value(&value);
        assert_eq!(result, "\"short\""); // JSON string representation
    }
}
