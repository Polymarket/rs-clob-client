//! Serde helpers for flexible deserialization.
//!
//! When the `tracing` feature is enabled, this module also logs warnings for any
//! unknown fields encountered during deserialization, helping detect API changes.

use serde::de::DeserializeOwned;
use serde_json::Value;

/// A `serde_as` type that deserializes strings or integers as `String`.
///
/// Use with `#[serde_as(as = "StringFromAny")]` for `String` fields
/// or `#[serde_as(as = "Option<StringFromAny>")]` for `Option<String>`.
pub struct StringFromAny;

impl<'de> serde_with::DeserializeAs<'de, String> for StringFromAny {
    fn deserialize_as<D>(deserializer: D) -> std::result::Result<String, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::fmt;

        use serde::de::{self, Visitor};

        struct StringOrNumberVisitor;

        impl Visitor<'_> for StringOrNumberVisitor {
            type Value = String;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string or integer")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.to_owned())
            }

            fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.to_string())
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.to_string())
            }
        }

        deserializer.deserialize_any(StringOrNumberVisitor)
    }
}

impl serde_with::SerializeAs<String> for StringFromAny {
    fn serialize_as<S>(source: &String, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(source)
    }
}

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

    tracing::trace!(
        type_name = %type_name::<T>(),
        json = %value,
        "deserializing JSON"
    );

    // Clone the value so we can look up unknown field values later
    let original = value.clone();

    // Collect unknown field paths during deserialization
    let mut unknown_paths: Vec<String> = Vec::new();

    let result: T = serde_ignored::deserialize(value, |path| {
        unknown_paths.push(path.to_string());
    })
    .inspect_err(|_| {
        // Re-deserialize with serde_path_to_error to get the error path
        let json_str = original.to_string();
        let jd = &mut serde_json::Deserializer::from_str(&json_str);
        let path_result: Result<T, _> = serde_path_to_error::deserialize(jd);
        if let Err(path_err) = path_result {
            let path = path_err.path().to_string();
            let inner_error = path_err.inner();
            let value_at_path = lookup_value(&original, &path);
            let value_display = format_value(value_at_path);

            tracing::error!(
                type_name = %type_name::<T>(),
                path = %path,
                value = %value_display,
                error = %inner_error,
                "deserialization failed"
            );
        }
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

/// Look up a value in a JSON structure by path.
///
/// Handles paths from both `serde_ignored` and `serde_path_to_error`:
/// - `?` for Option wrappers (skipped, as JSON has no Option representation)
/// - Numeric indices for arrays: `items.0` or `items[0]`
/// - Field names for objects: `foo.bar` or `foo.bar[0].baz`
///
/// Returns `None` if the path doesn't exist or traverses a non-container value.
#[cfg(feature = "tracing")]
fn lookup_value<'value>(value: &'value Value, path: &str) -> Option<&'value Value> {
    if path.is_empty() {
        return Some(value);
    }

    let mut current = value;

    // Parse path segments, handling both dot notation and bracket notation
    // e.g., "data[15].condition_id" -> ["data", "15", "condition_id"]
    let segments = parse_path_segments(path);

    for segment in segments {
        if segment.is_empty() || segment == "?" {
            continue;
        }

        match current {
            Value::Object(map) => {
                current = map.get(&segment)?;
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

/// Parse a path string into segments, handling both dot and bracket notation.
///
/// Examples:
/// - `"foo.bar"` -> `["foo", "bar"]`
/// - `"data[15].condition_id"` -> `["data", "15", "condition_id"]`
/// - `"items[0][1].value"` -> `["items", "0", "1", "value"]`
#[cfg(feature = "tracing")]
fn parse_path_segments(path: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();

    let mut chars = path.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '.' => {
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
            }
            '[' => {
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
                // Collect until closing bracket
                for inner in chars.by_ref() {
                    if inner == ']' {
                        break;
                    }
                    current.push(inner);
                }
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
            }
            ']' => {
                // Shouldn't happen if well-formed, but handle gracefully
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        segments.push(current);
    }

    segments
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

    // ========== StringFromAny tests ==========

    #[derive(Debug, Deserialize, PartialEq, serde::Serialize)]
    struct StringFromAnyStruct {
        #[serde(with = "serde_with::As::<StringFromAny>")]
        id: String,
    }

    #[derive(Debug, Deserialize, PartialEq, serde::Serialize)]
    struct OptionalStringFromAny {
        #[serde(with = "serde_with::As::<Option<StringFromAny>>")]
        id: Option<String>,
    }

    #[test]
    fn string_from_any_deserialize_string() {
        let json = serde_json::json!({ "id": "hello" });
        let result: StringFromAnyStruct =
            serde_json::from_value(json).expect("deserialization failed");
        assert_eq!(result.id, "hello");
    }

    #[test]
    fn string_from_any_deserialize_positive_integer() {
        let json = serde_json::json!({ "id": 12345 });
        let result: StringFromAnyStruct =
            serde_json::from_value(json).expect("deserialization failed");
        assert_eq!(result.id, "12345");
    }

    #[test]
    fn string_from_any_deserialize_negative_integer() {
        let json = serde_json::json!({ "id": -42 });
        let result: StringFromAnyStruct =
            serde_json::from_value(json).expect("deserialization failed");
        assert_eq!(result.id, "-42");
    }

    #[test]
    fn string_from_any_deserialize_zero() {
        let json = serde_json::json!({ "id": 0 });
        let result: StringFromAnyStruct =
            serde_json::from_value(json).expect("deserialization failed");
        assert_eq!(result.id, "0");
    }

    #[test]
    fn string_from_any_deserialize_large_u64() {
        // Test u64 max value
        let json = serde_json::json!({ "id": u64::MAX });
        let result: StringFromAnyStruct =
            serde_json::from_value(json).expect("deserialization failed");
        assert_eq!(result.id, u64::MAX.to_string());
    }

    #[test]
    fn string_from_any_deserialize_large_negative_i64() {
        // Test i64 min value
        let json = serde_json::json!({ "id": i64::MIN });
        let result: StringFromAnyStruct =
            serde_json::from_value(json).expect("deserialization failed");
        assert_eq!(result.id, i64::MIN.to_string());
    }

    #[test]
    fn string_from_any_serialize_back_to_string() {
        let obj = StringFromAnyStruct {
            id: "12345".to_owned(),
        };
        let json = serde_json::to_value(&obj).expect("serialization failed");
        assert_eq!(json, serde_json::json!({ "id": "12345" }));
    }

    #[test]
    fn string_from_any_roundtrip_from_string() {
        let json = serde_json::json!({ "id": "hello" });
        let obj: StringFromAnyStruct =
            serde_json::from_value(json).expect("deserialization failed");
        let back = serde_json::to_value(&obj).expect("serialization failed");
        assert_eq!(back, serde_json::json!({ "id": "hello" }));
    }

    #[test]
    fn string_from_any_roundtrip_from_integer() {
        let json = serde_json::json!({ "id": 42 });
        let obj: StringFromAnyStruct =
            serde_json::from_value(json).expect("deserialization failed");
        // After roundtrip, integer becomes string
        let back = serde_json::to_value(&obj).expect("serialization failed");
        assert_eq!(back, serde_json::json!({ "id": "42" }));
    }

    #[test]
    fn string_from_any_option_some_string() {
        let json = serde_json::json!({ "id": "hello" });
        let result: OptionalStringFromAny =
            serde_json::from_value(json).expect("deserialization failed");
        assert_eq!(result.id, Some("hello".to_owned()));
    }

    #[test]
    fn string_from_any_option_some_integer() {
        let json = serde_json::json!({ "id": 123 });
        let result: OptionalStringFromAny =
            serde_json::from_value(json).expect("deserialization failed");
        assert_eq!(result.id, Some("123".to_owned()));
    }

    #[test]
    fn string_from_any_option_none() {
        let json = serde_json::json!({ "id": null });
        let result: OptionalStringFromAny =
            serde_json::from_value(json).expect("deserialization failed");
        assert_eq!(result.id, None);
    }

    #[test]
    fn string_from_any_option_serialize_some() {
        let obj = OptionalStringFromAny {
            id: Some("test".to_owned()),
        };
        let json = serde_json::to_value(&obj).expect("serialization failed");
        assert_eq!(json, serde_json::json!({ "id": "test" }));
    }

    #[test]
    fn string_from_any_option_serialize_none() {
        let obj = OptionalStringFromAny { id: None };
        let json = serde_json::to_value(&obj).expect("serialization failed");
        assert_eq!(json, serde_json::json!({ "id": null }));
    }

    #[test]
    fn string_from_any_empty_string() {
        let json = serde_json::json!({ "id": "" });
        let result: StringFromAnyStruct =
            serde_json::from_value(json).expect("deserialization failed");
        assert_eq!(result.id, "");
    }

    // ========== lookup_value tests ==========

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

    #[cfg(feature = "tracing")]
    #[test]
    fn format_none_shows_placeholder() {
        let formatted = format_value(None);
        assert_eq!(formatted, "<unable to retrieve>");
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn lookup_option_marker_skipped() {
        // serde_ignored uses '?' for Option wrappers
        let json = serde_json::json!({"outer": {"inner": "value"}});
        // Path "?.outer.?.inner" should skip ? markers
        let result = lookup_value(&json, "?.outer.?.inner");
        assert_eq!(result, Some(&Value::String("value".to_owned())));
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
