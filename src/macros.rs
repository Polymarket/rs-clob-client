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
//! trace!(token_id = %token_id, "cache hit");
//! ```
//!
//! For unused variable suppression when tracing is disabled, use [`suppress!`]:
//!
//! ```ignore
//! trace!(token_id = %token_id, neg_risk = *neg_risk, "cache hit");
//! suppress!(token_id, neg_risk);
//! ```

/// Logs a message at the trace level, gated by `#[cfg(feature = "tracing")]`.
///
/// Accepts the same arguments as `tracing::trace!`.
#[doc(hidden)]
#[macro_export]
macro_rules! trace {
    ($($args:tt)*) => {{
        #[cfg(feature = "tracing")]
        tracing::trace!($($args)*);
    }};
}

/// Logs a message at the debug level, gated by `#[cfg(feature = "tracing")]`.
///
/// Accepts the same arguments as `tracing::debug!`.
#[doc(hidden)]
#[macro_export]
macro_rules! debug {
    ($($args:tt)*) => {{
        #[cfg(feature = "tracing")]
        tracing::debug!($($args)*);
    }};
}

/// Logs a message at the warn level, gated by `#[cfg(feature = "tracing")]`.
///
/// Accepts the same arguments as `tracing::warn!`.
#[doc(hidden)]
#[macro_export]
macro_rules! warn {
    ($($args:tt)*) => {{
        #[cfg(feature = "tracing")]
        tracing::warn!($($args)*);
    }};
}

/// Logs a message at the error level, gated by `#[cfg(feature = "tracing")]`.
///
/// Accepts the same arguments as `tracing::error!`.
#[doc(hidden)]
#[macro_export]
macro_rules! error {
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
/// trace!(token_id = %token_id, neg_risk = *neg_risk, "cache hit");
/// suppress!(token_id, neg_risk);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! suppress {
    ($($var:expr),* $(,)?) => {
        #[cfg(not(feature = "tracing"))]
        {
            $(let _ = &$var;)*
        }
    };
}

#[cfg(test)]
mod tests {

    #[test]
    fn log_macros_compile_and_run() {
        let token_id = "test_token";
        let value = 42;

        // All macros should compile and not panic
        trace!(token_id = %token_id, value = value, "trace message");
        debug!(token_id = %token_id, "debug message");
        warn!("warn message");
        error!("error message");
        suppress!(token_id, value);
    }

    #[test]
    fn suppress_compiles() {
        let token_id = "test_token";
        let neg_risk = true;

        trace!(token_id = %token_id, neg_risk = neg_risk, "cache hit");
        suppress!(token_id, neg_risk);
    }

    #[test]
    fn log_macros_with_format_specifiers() {
        let id = "abc123";
        let count: usize = 10;
        let data = vec![1, 2, 3];

        // Display format
        trace!(%id, "with display");
        // Debug format
        debug!(?data, "with debug");
        // Direct value
        warn!(count, "with value");
        suppress!(id, count, data);
    }
}
