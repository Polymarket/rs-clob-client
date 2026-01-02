//! Deserialization with unknown field warnings.
//!
//! When the `tracing` feature is enabled, this module logs warnings for any
//! unknown fields encountered during deserialization, helping detect API changes.

use serde::de::DeserializeOwned;
use serde_json::Value;

/// Deserialize JSON with unknown field warnings.
///
/// This function deserializes JSON to a target type while detecting and logging
/// any fields that are not captured by the type definition.
///
/// # Arguments
///
/// * `value` - The JSON value to deserialize
///
/// # Returns
///
/// The deserialized value, or an error if deserialization fails.
/// Unknown fields trigger warnings but do not cause deserialization to fail.
///
/// # Example
///
/// ```ignore
/// let json = serde_json::json!({
///     "known_field": "value",
///     "unknown_field": "extra"
/// });
/// let result: MyType = deserialize_with_warnings(json)?;
/// // Logs: WARN Unknown field "unknown_field" with value "extra" in MyType
/// ```
#[cfg(feature = "tracing")]
pub fn deserialize_with_warnings<T: DeserializeOwned>(value: Value) -> crate::Result<T> {
    use std::any::type_name;

    // Clone the value so we can look up unknown field values later
    let original = value.clone();

    // Collect unknown field paths during deserialization
    let mut unknown_paths: Vec<String> = Vec::new();

    let result: T = serde_ignored::deserialize(value, |path| {
        unknown_paths.push(path.to_string());
    })?;

    // Log warnings for unknown fields with their values
    if !unknown_paths.is_empty() {
        let type_name = type_name::<T>();
        for path in unknown_paths {
            let field_value = lookup_value(&original, &path);
            let value_display = format_value(field_value);

            tracing::warn!(
                type_name = %type_name,
                field = %path,
                value = %value_display,
                "unknown field in API response"
            );
        }
    }

    Ok(result)
}

/// Pass-through deserialization when tracing is disabled.
#[cfg(not(feature = "tracing"))]
pub fn deserialize_with_warnings<T: DeserializeOwned>(value: Value) -> crate::Result<T> {
    Ok(serde_json::from_value(value)?)
}

/// Look up a value in a JSON structure by dot-separated path.
///
/// Handles paths from `serde_ignored` which use:
/// - `?` for Option wrappers (skipped, as JSON has no Option representation)
/// - Numeric indices for arrays (e.g., `0`, `1`)
/// - Field names for objects
///
/// Returns `None` if the path doesn't exist or traverses a non-container value.
#[cfg(feature = "tracing")]
fn lookup_value<'value>(value: &'value Value, path: &str) -> Option<&'value Value> {
    if path.is_empty() {
        return Some(value);
    }

    let mut current = value;

    // Filter empty segments and skip `?` (Option marker from serde_ignored)
    for segment in path.split('.').filter(|s| !s.is_empty() && *s != "?") {
        match current {
            Value::Object(map) => {
                // Try as object key first, then as array index if current is actually an array
                current = map.get(segment)?;
            }
            Value::Array(arr) => {
                let index: usize = segment.parse().ok()?;
                current = arr.get(index)?;
            }
            _ => return None,
        }
    }

    Some(current)
}

/// Format a JSON value for logging.
#[cfg(feature = "tracing")]
fn format_value(value: Option<&Value>) -> String {
    match value {
        Some(v) => v.to_string(),
        None => "<unable to retrieve>".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestStruct {
        known_field: String,
        #[serde(default)]
        optional_field: Option<i32>,
    }

    #[test]
    fn deserialize_known_fields_only() {
        let json = serde_json::json!({
            "known_field": "value",
            "optional_field": 42
        });

        let result: TestStruct = deserialize_with_warnings(json).expect("deserialization failed");
        assert_eq!(result.known_field, "value");
        assert_eq!(result.optional_field, Some(42));
    }

    #[test]
    fn deserialize_with_unknown_fields() {
        let json = serde_json::json!({
            "known_field": "value",
            "unknown_field": "extra",
            "another_unknown": 123
        });

        // Should succeed - extra fields are logged but not an error
        let result: TestStruct = deserialize_with_warnings(json).expect("deserialization failed");
        assert_eq!(result.known_field, "value");
        assert_eq!(result.optional_field, None);
    }

    #[test]
    fn deserialize_missing_required_field_fails() {
        let json = serde_json::json!({
            "optional_field": 42
        });

        let result: crate::Result<TestStruct> = deserialize_with_warnings(json);
        result.unwrap_err();
    }

    #[test]
    fn deserialize_array() {
        let json = serde_json::json!([1, 2, 3]);

        let result: Vec<i32> = deserialize_with_warnings(json).expect("deserialization failed");
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct NestedStruct {
        outer: String,
        inner: InnerStruct,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct InnerStruct {
        value: i32,
    }

    #[test]
    fn deserialize_nested_unknown_fields() {
        let json = serde_json::json!({
            "outer": "test",
            "inner": {
                "value": 42,
                "nested_unknown": "surprise"
            }
        });

        let result: NestedStruct = deserialize_with_warnings(json).expect("deserialization failed");
        assert_eq!(result.outer, "test");
        assert_eq!(result.inner.value, 42);
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn lookup_simple_path() {
        let json = serde_json::json!({
            "foo": "bar"
        });

        let result = lookup_value(&json, "foo");
        assert_eq!(result, Some(&Value::String("bar".to_owned())));
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn lookup_nested_path() {
        let json = serde_json::json!({
            "outer": {
                "inner": "value"
            }
        });

        let result = lookup_value(&json, "outer.inner");
        assert_eq!(result, Some(&Value::String("value".to_owned())));
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn lookup_array_index() {
        let json = serde_json::json!({
            "items": ["a", "b", "c"]
        });

        let result = lookup_value(&json, "items.1");
        assert_eq!(result, Some(&Value::String("b".to_owned())));
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn lookup_empty_path_returns_root() {
        let json = serde_json::json!({"foo": "bar"});
        let result = lookup_value(&json, "");
        assert_eq!(result, Some(&json));
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn lookup_consecutive_dots_handled() {
        let json = serde_json::json!({"foo": {"bar": "value"}});
        // Path "foo..bar" should skip the empty segment and find "foo.bar"
        let result = lookup_value(&json, "foo..bar");
        assert_eq!(result, Some(&Value::String("value".to_owned())));
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn lookup_leading_dot_handled() {
        let json = serde_json::json!({"foo": "bar"});
        // Path ".foo" should skip the leading empty segment
        let result = lookup_value(&json, ".foo");
        assert_eq!(result, Some(&Value::String("bar".to_owned())));
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn lookup_invalid_array_index_returns_none() {
        let json = serde_json::json!({"items": [1, 2, 3]});
        let result = lookup_value(&json, "items.abc");
        assert_eq!(result, None);
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn lookup_array_out_of_bounds_returns_none() {
        let json = serde_json::json!({"items": [1, 2, 3]});
        let result = lookup_value(&json, "items.100");
        assert_eq!(result, None);
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn lookup_through_primitive_returns_none() {
        let json = serde_json::json!({"foo": "bar"});
        // Can't traverse through a string
        let result = lookup_value(&json, "foo.baz");
        assert_eq!(result, None);
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn format_shows_full_string() {
        let long_string = "a".repeat(300);
        let value = Value::String(long_string.clone());

        let formatted = format_value(Some(&value));
        // Full JSON string with quotes
        assert_eq!(formatted, format!("\"{long_string}\""));
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn format_array_shows_full_json() {
        let value = serde_json::json!([1, 2, 3, 4, 5]);

        let formatted = format_value(Some(&value));
        assert_eq!(formatted, "[1,2,3,4,5]");
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn format_object_shows_full_json() {
        let value = serde_json::json!({"a": 1, "b": 2});

        let formatted = format_value(Some(&value));
        // JSON object serialization order may vary, check both keys present
        assert!(formatted.contains("\"a\":1"));
        assert!(formatted.contains("\"b\":2"));
    }

    /// Test that verifies warnings are actually emitted for unknown fields.
    /// This test captures tracing output to prove the feature works.
    #[cfg(feature = "tracing")]
    #[test]
    fn warning_is_emitted_for_unknown_fields() {
        use std::sync::{Arc, Mutex};

        use tracing_subscriber::layer::SubscriberExt as _;

        // Capture warnings in a buffer
        let warnings: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let warnings_clone = Arc::clone(&warnings);

        // Custom layer that captures warn events
        let layer = tracing_subscriber::fmt::layer()
            .with_writer(move || {
                struct CaptureWriter(Arc<Mutex<Vec<String>>>);
                impl std::io::Write for CaptureWriter {
                    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                        if let Ok(s) = std::str::from_utf8(buf) {
                            self.0.lock().expect("lock").push(s.to_owned());
                        }
                        Ok(buf.len())
                    }
                    fn flush(&mut self) -> std::io::Result<()> {
                        Ok(())
                    }
                }
                CaptureWriter(Arc::clone(&warnings_clone))
            })
            .with_ansi(false);

        let subscriber = tracing_subscriber::registry().with(layer);

        // Run the deserialization with our subscriber
        tracing::subscriber::with_default(subscriber, || {
            let json = serde_json::json!({
                "known_field": "value",
                "secret_new_field": "surprise!",
                "another_unknown": 42
            });

            let result: TestStruct =
                deserialize_with_warnings(json).expect("deserialization should succeed");
            assert_eq!(result.known_field, "value");
        });

        // Check that warnings were captured
        let captured = warnings.lock().expect("lock");
        let all_output = captured.join("");

        assert!(
            all_output.contains("unknown field"),
            "Expected 'unknown field' in output, got: {all_output}"
        );
        assert!(
            all_output.contains("secret_new_field"),
            "Expected 'secret_new_field' in output, got: {all_output}"
        );
    }
}
