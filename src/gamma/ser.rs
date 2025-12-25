//! Custom serialization helpers for Gamma API request types.

use chrono::{DateTime, Utc};
use serde::Serializer;

/// Serialize `Vec<T>` as comma-separated string.
#[expect(
    clippy::ref_option,
    reason = "serde serialize_with requires &Option<T>"
)]
pub fn comma_separated<T, S>(v: &Option<Vec<T>>, s: S) -> Result<S::Ok, S::Error>
where
    T: ToString,
    S: Serializer,
{
    match v {
        Some(vec) if !vec.is_empty() => s.serialize_str(
            &vec.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        ),
        _ => s.serialize_none(),
    }
}

/// Serialize `DateTime` as RFC3339 string.
#[expect(
    clippy::ref_option,
    reason = "serde serialize_with requires &Option<T>"
)]
pub fn rfc3339<S>(v: &Option<DateTime<Utc>>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match v {
        Some(dt) => s.serialize_str(&dt.to_rfc3339()),
        None => s.serialize_none(),
    }
}

/// Helper to skip empty `Vec` wrapped in `Option` during serialization.
#[expect(
    clippy::ref_option,
    reason = "serde skip_serializing_if requires &Option<T>"
)]
pub fn is_empty_vec<T>(opt: &Option<Vec<T>>) -> bool {
    match opt {
        Some(v) => v.is_empty(),
        None => true,
    }
}
