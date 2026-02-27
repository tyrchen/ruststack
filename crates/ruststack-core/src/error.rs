//! Error types for the RustStack core.

/// Core error type for RustStack infrastructure.
#[derive(Debug, thiserror::Error)]
pub enum RustStackError {
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

/// Convenience result type for RustStack operations.
pub type RustStackResult<T> = Result<T, RustStackError>;
