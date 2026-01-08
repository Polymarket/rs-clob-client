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

    use serde::de::Error as _;

    tracing::trace!(
        type_name = %type_name::<T>(),
        json = %value,
        "deserializing JSON"
    );

    // Clone the value so we can look up unknown field values later
    let original = value.clone();

    // Collect unknown field paths during deserialization
    let mut unknown_paths: Vec<String> = Vec::new();

    let result: Result<T, _> = serde_ignored::deserialize(value, |path| {
        unknown_paths.push(path.to_string());
    });

    if let Ok(value) = result {
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
        Ok(value)
    } else {
        // Re-deserialize with serde_path_to_error to get detailed error path
        let json_str = original.to_string();
        let mut jd = serde_json::Deserializer::from_str(&json_str);
        serde_path_to_error::deserialize(&mut jd)
            .inspect_err(|e| {
                let field_value = lookup_value(&original, &e.path().to_string());
                tracing::error!(
                    type_name = %type_name::<T>(),
                    field = %e.path(),
                    value = %format_value(field_value),
                    error = %e.inner(),
                    "deserialization failed"
                );
            })
            .map_err(|e| crate::Error::from(serde_json::Error::custom(e.to_string())))
    }
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
#[expect(clippy::string_slice, reason = "JSON paths are ASCII, safe to slice")]
fn lookup_value<'value>(value: &'value Value, path: &str) -> Option<&'value Value> {
    if path.is_empty() {
        return Some(value);
    }

    let mut current = value;

    // Parse path like "[0].asset", "foo.bar[2].baz", or "?.0.field" (serde_ignored format)
    let mut remaining = path;
    while !remaining.is_empty() {
        // Skip leading dots
        remaining = remaining.trim_start_matches('.');

        if remaining.starts_with('[') {
            // Bracket notation array index: [0]
            let end = remaining.find(']')?;
            let index: usize = remaining[1..end].parse().ok()?;
            current = current.as_array()?.get(index)?;
            remaining = &remaining[end + 1..];
        } else {
            // Object key or dot notation array index
            let end = remaining.find(['.', '[']).unwrap_or(remaining.len());
            let key = &remaining[..end];
            if !key.is_empty() && key != "?" {
                // Try as array index first (for serde_ignored format like "?.0")
                if let Ok(index) = key.parse::<usize>() {
                    if let Some(arr) = current.as_array() {
                        current = arr.get(index)?;
                    } else {
                        // Fall back to object key
                        current = current.as_object()?.get(key)?;
                    }
                } else {
                    current = current.as_object()?.get(key)?;
                }
            }
            remaining = &remaining[end..];
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

    /// Test that verifies errors show field path and value when deserialization fails.
    #[cfg(feature = "tracing")]
    #[test]
    fn error_shows_field_path_and_value() {
        use std::sync::{Arc, Mutex};

        use tracing_subscriber::layer::SubscriberExt as _;

        #[expect(dead_code, reason = "fields only used for deserialization test")]
        #[derive(Debug, Deserialize)]
        struct TypeWithInt {
            name: String,
            count: i32, // This field expects an integer
        }

        // Capture errors in a buffer
        let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let errors_clone = Arc::clone(&errors);

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
                CaptureWriter(Arc::clone(&errors_clone))
            })
            .with_ansi(false);

        let subscriber = tracing_subscriber::registry().with(layer);

        // Run the deserialization with our subscriber
        tracing::subscriber::with_default(subscriber, || {
            let json = serde_json::json!({
                "name": "test",
                "count": "not_a_number"  // Wrong type - will fail
            });

            let result: crate::Result<TypeWithInt> = deserialize_with_warnings(json);
            assert!(result.is_err(), "deserialization should fail");
        });

        // Check that error log was captured with field path and value
        let captured = errors.lock().expect("lock");
        let all_output = captured.join("");

        assert!(
            all_output.contains("deserialization failed"),
            "Expected 'deserialization failed' in output, got: {all_output}"
        );
        assert!(
            all_output.contains("count"),
            "Expected field path 'count' in output, got: {all_output}"
        );
        assert!(
            all_output.contains("not_a_number"),
            "Expected field value 'not_a_number' in output, got: {all_output}"
        );
    }

    /// Test error reporting for nested fields in arrays.
    #[cfg(feature = "tracing")]
    #[test]
    fn error_shows_nested_array_path() {
        use std::sync::{Arc, Mutex};

        use tracing_subscriber::layer::SubscriberExt as _;

        #[expect(dead_code, reason = "fields only used for deserialization test")]
        #[derive(Debug, Deserialize)]
        struct Item {
            id: i32,
        }

        #[expect(dead_code, reason = "fields only used for deserialization test")]
        #[derive(Debug, Deserialize)]
        struct Container {
            items: Vec<Item>,
        }

        let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let errors_clone = Arc::clone(&errors);

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
                CaptureWriter(Arc::clone(&errors_clone))
            })
            .with_ansi(false);

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            let json = serde_json::json!({
                "items": [
                    {"id": 1},
                    {"id": "bad"},  // Second item has wrong type
                    {"id": 3}
                ]
            });

            let result: crate::Result<Container> = deserialize_with_warnings(json);
            assert!(result.is_err(), "deserialization should fail");
        });

        let captured = errors.lock().expect("lock");
        let all_output = captured.join("");

        assert!(
            all_output.contains("deserialization failed"),
            "Expected 'deserialization failed' in output, got: {all_output}"
        );
        // Should show path to the failing field, like "items[1].id"
        assert!(
            all_output.contains("items") && all_output.contains('1') && all_output.contains("id"),
            "Expected path containing 'items', '1', and 'id' in output, got: {all_output}"
        );
        assert!(
            all_output.contains("bad"),
            "Expected field value 'bad' in output, got: {all_output}"
        );
    }
}
