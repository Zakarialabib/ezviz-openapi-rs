//! Error types for the EZVIZ Open Platform client.

use thiserror::Error;

/// Unified error type.
#[derive(Debug, Error)]
pub enum EzvizError {
    /// The EZVIZ platform returned a non-`200` `code` in the JSON envelope.
    #[error("EZVIZ API error {code}: {msg}")]
    Api {
        /// Platform status code from the JSON envelope.
        code: String,
        /// Human-readable message from the JSON envelope.
        msg: String,
    },

    /// Transport-level failure (DNS, TLS, connect, timeout).
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON decode failure of a response or request body.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Filesystem / IO error (PC SDK loading, etc.).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// PCOpenSDK native library reported failure (feature `pc-sdk`).
    #[cfg(feature = "pc-sdk")]
    #[error("PCOpenSDK error: {0}")]
    Sdk(String),
}

/// Crate-wide result alias using [`EzvizError`].
pub type Result<T> = std::result::Result<T, EzvizError>;