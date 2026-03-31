//! Error types for the Rustack core.

/// Core error type for Rustack infrastructure.
#[derive(Debug, thiserror::Error)]
pub enum RustackError {
    /// Invalid AWS account ID format.
    #[error("invalid AWS account ID: {0} (must be 12-digit numeric string)")]
    InvalidAccountId(String),

    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Internal error with context.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

/// Convenience result type for Rustack operations.
pub type RustackResult<T> = Result<T, RustackError>;
