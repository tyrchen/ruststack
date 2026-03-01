//! DynamoDB error types.
//!
//! DynamoDB errors use JSON format with a `__type` field containing the
//! fully-qualified error type name.

use std::fmt;

/// Well-known DynamoDB error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum DynamoDBErrorCode {
    /// Table already exists.
    ResourceInUseException,
    /// Table not found.
    ResourceNotFoundException,
    /// Condition check failed.
    ConditionalCheckFailedException,
    /// Transaction canceled.
    TransactionCanceledException,
    /// Transaction conflict.
    TransactionConflictException,
    /// Transaction in progress.
    TransactionInProgressException,
    /// Idempotent parameter mismatch.
    IdempotentParameterMismatchException,
    /// Item collection size limit exceeded.
    ItemCollectionSizeLimitExceededException,
    /// Provisioned throughput exceeded.
    ProvisionedThroughputExceededException,
    /// Request limit exceeded.
    RequestLimitExceeded,
    /// Validation error.
    #[default]
    ValidationException,
    /// Serialization error.
    SerializationException,
    /// Internal server error.
    InternalServerError,
    /// Missing action.
    MissingAction,
    /// Access denied.
    AccessDeniedException,
    /// Unknown operation.
    UnrecognizedClientException,
}

impl DynamoDBErrorCode {
    /// Returns the fully-qualified error type string for JSON `__type` field.
    #[must_use]
    pub fn error_type(&self) -> &'static str {
        match self {
            Self::ResourceInUseException => {
                "com.amazonaws.dynamodb.v20120810#ResourceInUseException"
            }
            Self::ResourceNotFoundException => {
                "com.amazonaws.dynamodb.v20120810#ResourceNotFoundException"
            }
            Self::ConditionalCheckFailedException => {
                "com.amazonaws.dynamodb.v20120810#ConditionalCheckFailedException"
            }
            Self::TransactionCanceledException => {
                "com.amazonaws.dynamodb.v20120810#TransactionCanceledException"
            }
            Self::TransactionConflictException => {
                "com.amazonaws.dynamodb.v20120810#TransactionConflictException"
            }
            Self::TransactionInProgressException => {
                "com.amazonaws.dynamodb.v20120810#TransactionInProgressException"
            }
            Self::IdempotentParameterMismatchException => {
                "com.amazonaws.dynamodb.v20120810#IdempotentParameterMismatchException"
            }
            Self::ItemCollectionSizeLimitExceededException => {
                "com.amazonaws.dynamodb.v20120810#ItemCollectionSizeLimitExceededException"
            }
            Self::ProvisionedThroughputExceededException => {
                "com.amazonaws.dynamodb.v20120810#ProvisionedThroughputExceededException"
            }
            Self::RequestLimitExceeded => "com.amazonaws.dynamodb.v20120810#RequestLimitExceeded",
            Self::ValidationException => "com.amazon.coral.validate#ValidationException",
            Self::SerializationException => {
                "com.amazonaws.dynamodb.v20120810#SerializationException"
            }
            Self::InternalServerError => "com.amazonaws.dynamodb.v20120810#InternalServerError",
            Self::MissingAction => "com.amazonaws.dynamodb.v20120810#MissingAction",
            Self::AccessDeniedException => "com.amazonaws.dynamodb.v20120810#AccessDeniedException",
            Self::UnrecognizedClientException => {
                "com.amazonaws.dynamodb.v20120810#UnrecognizedClientException"
            }
        }
    }

    /// Returns the short error code string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ResourceInUseException => "ResourceInUseException",
            Self::ResourceNotFoundException => "ResourceNotFoundException",
            Self::ConditionalCheckFailedException => "ConditionalCheckFailedException",
            Self::TransactionCanceledException => "TransactionCanceledException",
            Self::TransactionConflictException => "TransactionConflictException",
            Self::TransactionInProgressException => "TransactionInProgressException",
            Self::IdempotentParameterMismatchException => "IdempotentParameterMismatchException",
            Self::ItemCollectionSizeLimitExceededException => {
                "ItemCollectionSizeLimitExceededException"
            }
            Self::ProvisionedThroughputExceededException => {
                "ProvisionedThroughputExceededException"
            }
            Self::RequestLimitExceeded => "RequestLimitExceeded",
            Self::ValidationException => "ValidationException",
            Self::SerializationException => "SerializationException",
            Self::InternalServerError => "InternalServerError",
            Self::MissingAction => "MissingAction",
            Self::AccessDeniedException => "AccessDeniedException",
            Self::UnrecognizedClientException => "UnrecognizedClientException",
        }
    }

    /// Returns the default HTTP status code for this error.
    #[must_use]
    pub fn default_status_code(&self) -> http::StatusCode {
        match self {
            Self::InternalServerError => http::StatusCode::INTERNAL_SERVER_ERROR,
            _ => http::StatusCode::BAD_REQUEST,
        }
    }
}

impl fmt::Display for DynamoDBErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A DynamoDB error response.
#[derive(Debug)]
pub struct DynamoDBError {
    /// The error code.
    pub code: DynamoDBErrorCode,
    /// A human-readable error message.
    pub message: String,
    /// The HTTP status code.
    pub status_code: http::StatusCode,
    /// The underlying source error, if any.
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for DynamoDBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamoDBError({}): {}", self.code, self.message)
    }
}

impl std::error::Error for DynamoDBError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl DynamoDBError {
    /// Create a new `DynamoDBError` from an error code.
    #[must_use]
    pub fn new(code: DynamoDBErrorCode) -> Self {
        Self {
            status_code: code.default_status_code(),
            message: code.as_str().to_owned(),
            code,
            source: None,
        }
    }

    /// Create a new `DynamoDBError` with a custom message.
    #[must_use]
    pub fn with_message(code: DynamoDBErrorCode, message: impl Into<String>) -> Self {
        Self {
            status_code: code.default_status_code(),
            message: message.into(),
            code,
            source: None,
        }
    }

    /// Set the source error.
    #[must_use]
    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Returns the `__type` string for the JSON error response.
    #[must_use]
    pub fn error_type(&self) -> &'static str {
        self.code.error_type()
    }

    // -- Convenience constructors --

    /// Table already exists.
    #[must_use]
    pub fn resource_in_use(message: impl Into<String>) -> Self {
        Self::with_message(DynamoDBErrorCode::ResourceInUseException, message)
    }

    /// Table or resource not found.
    #[must_use]
    pub fn resource_not_found(message: impl Into<String>) -> Self {
        Self::with_message(DynamoDBErrorCode::ResourceNotFoundException, message)
    }

    /// Condition expression evaluated to false.
    #[must_use]
    pub fn conditional_check_failed(message: impl Into<String>) -> Self {
        Self::with_message(DynamoDBErrorCode::ConditionalCheckFailedException, message)
    }

    /// Validation error.
    #[must_use]
    pub fn validation(message: impl Into<String>) -> Self {
        Self::with_message(DynamoDBErrorCode::ValidationException, message)
    }

    /// Serialization error.
    #[must_use]
    pub fn serialization_exception(message: impl Into<String>) -> Self {
        Self::with_message(DynamoDBErrorCode::SerializationException, message)
    }

    /// Internal server error.
    #[must_use]
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::with_message(DynamoDBErrorCode::InternalServerError, message)
    }

    /// Missing action header.
    #[must_use]
    pub fn missing_action() -> Self {
        Self::with_message(
            DynamoDBErrorCode::MissingAction,
            "Missing required header: X-Amz-Target",
        )
    }

    /// Unknown operation.
    #[must_use]
    pub fn unknown_operation(target: &str) -> Self {
        Self::with_message(
            DynamoDBErrorCode::UnrecognizedClientException,
            format!("Unrecognized operation: {target}"),
        )
    }
}

/// Create a `DynamoDBError` from an error code.
///
/// # Examples
///
/// ```
/// use ruststack_dynamodb_model::dynamodb_error;
/// use ruststack_dynamodb_model::error::DynamoDBErrorCode;
///
/// let err = dynamodb_error!(ValidationException);
/// assert_eq!(err.code, DynamoDBErrorCode::ValidationException);
///
/// let err = dynamodb_error!(ResourceNotFoundException, "Table not found");
/// assert_eq!(err.message, "Table not found");
/// ```
#[macro_export]
macro_rules! dynamodb_error {
    ($code:ident) => {
        $crate::error::DynamoDBError::new($crate::error::DynamoDBErrorCode::$code)
    };
    ($code:ident, $msg:expr) => {
        $crate::error::DynamoDBError::with_message($crate::error::DynamoDBErrorCode::$code, $msg)
    };
}
