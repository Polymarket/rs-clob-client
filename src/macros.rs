//! Ergonomic logging macros that wrap `tracing` with `#[cfg(feature = "tracing")]`.
//!
//! These macros eliminate the repetitive `#[cfg(feature = "tracing")]` boilerplate:
//!
//! ```ignore
//! // Before:
//! #[cfg(feature = "tracing")]
//! tracing::trace!(token_id = %token_id, "cache hit");
//!
//! // After:
//! log_trace!(token_id = %token_id, "cache hit");
//! ```
//!
//! For unused variable suppression when tracing is disabled, use [`log_suppress!`]:
//!
//! ```ignore
//! log_trace!(token_id = %token_id, neg_risk = *neg_risk, "cache hit");
//! log_suppress!(token_id, neg_risk);
//! ```

/// Logs a message at the trace level, gated by `#[cfg(feature = "tracing")]`.
///
/// Accepts the same arguments as `tracing::trace!`.
macro_rules! log_trace {
    ($($args:tt)*) => {{
        #[cfg(feature = "tracing")]
        tracing::trace!($($args)*);
    }};
}

/// Logs a message at the debug level, gated by `#[cfg(feature = "tracing")]`.
///
/// Accepts the same arguments as `tracing::debug!`.
macro_rules! log_debug {
    ($($args:tt)*) => {{
        #[cfg(feature = "tracing")]
        tracing::debug!($($args)*);
    }};
}

/// Logs a message at the warn level, gated by `#[cfg(feature = "tracing")]`.
///
/// Accepts the same arguments as `tracing::warn!`.
macro_rules! log_warn {
    ($($args:tt)*) => {{
        #[cfg(feature = "tracing")]
        tracing::warn!($($args)*);
    }};
}

/// Logs a message at the error level, gated by `#[cfg(feature = "tracing")]`.
///
/// Accepts the same arguments as `tracing::error!`.
macro_rules! log_error {
    ($($args:tt)*) => {{
        #[cfg(feature = "tracing")]
        tracing::error!($($args)*);
    }};
}

/// Suppresses unused variable warnings when tracing is disabled.
///
/// When the `tracing` feature is disabled, variables used only in log statements
/// would trigger unused variable warnings. This macro creates references to
/// suppress those warnings without any runtime cost.
///
/// # Example
///
/// ```ignore
/// log_trace!(token_id = %token_id, neg_risk = *neg_risk, "cache hit");
/// log_suppress!(token_id, neg_risk);
/// ```
macro_rules! log_suppress {
    ($($var:expr),* $(,)?) => {
        #[cfg(not(feature = "tracing"))]
        {
            $(let _ = &$var;)*
        }
    };
}

pub(crate) use log_debug;
pub(crate) use log_error;
pub(crate) use log_suppress;
pub(crate) use log_trace;
pub(crate) use log_warn;
