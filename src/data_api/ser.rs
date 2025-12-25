//! Custom serialization helpers for Data API request types.

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

/// Serialize `Vec<T>` (non-optional) as comma-separated string.
pub fn comma_separated_vec<T, S>(v: &[T], s: S) -> Result<S::Ok, S::Error>
where
    T: ToString,
    S: Serializer,
{
    if v.is_empty() {
        s.serialize_none()
    } else {
        s.serialize_str(
            &v.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        )
    }
}

/// Helper to skip empty `Vec` during serialization.
pub fn vec_is_empty<T>(v: &[T]) -> bool {
    v.is_empty()
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
