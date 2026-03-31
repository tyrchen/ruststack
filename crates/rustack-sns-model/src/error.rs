//! SNS error types.
//!
//! SNS errors use XML format with an `<Error>` element containing
//! `<Code>`, `<Message>`, and `<Type>` (fault) fields.

use std::fmt;

/// Well-known SNS error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum SnsErrorCode {
    /// Resource not found.
    #[default]
    NotFound,
    /// Invalid parameter.
    InvalidParameter,
    /// Invalid parameter value.
    InvalidParameterValue,
    /// Authorization error.
    AuthorizationError,
    /// Internal server error.
    InternalError,
    /// Request was throttled.
    Throttled,
    /// Subscription limit exceeded.
    SubscriptionLimitExceeded,
    /// Topic limit exceeded.
    TopicLimitExceeded,
    /// Filter policy limit exceeded.
    FilterPolicyLimitExceeded,
    /// Invalid security token or credentials.
    InvalidSecurity,
    /// Endpoint is disabled.
    EndpointDisabled,
    /// Platform application is disabled.
    PlatformApplicationDisabled,
    /// Tag policy violation.
    TagPolicy,
    /// Tag limit exceeded.
    TagLimitExceeded,
    /// Validation exception.
    ValidationException,
    /// Duplicate IDs in batch request.
    BatchEntryIdsNotDistinct,
    /// Empty batch request.
    EmptyBatchRequest,
    /// Too many entries in batch request.
    TooManyEntriesInBatchRequest,
    /// Missing action parameter.
    MissingAction,
}

impl SnsErrorCode {
    /// The error code string used in the XML `<Code>` element.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound => "NotFound",
            Self::InvalidParameter => "InvalidParameter",
            Self::InvalidParameterValue => "InvalidParameterValue",
            Self::AuthorizationError => "AuthorizationError",
            Self::InternalError => "InternalError",
            Self::Throttled => "Throttled",
            Self::SubscriptionLimitExceeded => "SubscriptionLimitExceeded",
            Self::TopicLimitExceeded => "TopicLimitExceeded",
            Self::FilterPolicyLimitExceeded => "FilterPolicyLimitExceeded",
            Self::InvalidSecurity => "InvalidSecurity",
            Self::EndpointDisabled => "EndpointDisabled",
            Self::PlatformApplicationDisabled => "PlatformApplicationDisabled",
            Self::TagPolicy => "TagPolicy",
            Self::TagLimitExceeded => "TagLimitExceeded",
            Self::ValidationException => "ValidationException",
            Self::BatchEntryIdsNotDistinct => "BatchEntryIdsNotDistinct",
            Self::EmptyBatchRequest => "EmptyBatchRequest",
            Self::TooManyEntriesInBatchRequest => "TooManyEntriesInBatchRequest",
            Self::MissingAction => "MissingAction",
        }
    }

    /// HTTP status code for this error.
    #[must_use]
    pub fn status_code(&self) -> http::StatusCode {
        match self {
            Self::NotFound => http::StatusCode::NOT_FOUND,
            Self::AuthorizationError
            | Self::SubscriptionLimitExceeded
            | Self::TopicLimitExceeded
            | Self::FilterPolicyLimitExceeded
            | Self::InvalidSecurity => http::StatusCode::FORBIDDEN,
            Self::InternalError => http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::Throttled => http::StatusCode::TOO_MANY_REQUESTS,
            Self::InvalidParameter
            | Self::InvalidParameterValue
            | Self::EndpointDisabled
            | Self::PlatformApplicationDisabled
            | Self::TagPolicy
            | Self::TagLimitExceeded
            | Self::ValidationException
            | Self::BatchEntryIdsNotDistinct
            | Self::EmptyBatchRequest
            | Self::TooManyEntriesInBatchRequest
            | Self::MissingAction => http::StatusCode::BAD_REQUEST,
        }
    }

    /// Fault type: `"Sender"` for client errors, `"Receiver"` for server errors.
    #[must_use]
    pub fn fault(&self) -> &'static str {
        match self {
            Self::InternalError | Self::Throttled => "Receiver",
            _ => "Sender",
        }
    }
}

impl fmt::Display for SnsErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// An SNS error response.
#[derive(Debug)]
pub struct SnsError {
    /// The error code.
    pub code: SnsErrorCode,
    /// A human-readable error message.
    pub message: String,
}

impl fmt::Display for SnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SnsError({}): {}", self.code, self.message)
    }
}

impl std::error::Error for SnsError {}

impl SnsError {
    /// Create a new `SnsError` with a code and message.
    #[must_use]
    pub fn new(code: SnsErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    // -- Convenience constructors --

    /// Resource not found.
    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(SnsErrorCode::NotFound, message)
    }

    /// Invalid parameter.
    #[must_use]
    pub fn invalid_parameter(message: impl Into<String>) -> Self {
        Self::new(SnsErrorCode::InvalidParameter, message)
    }

    /// Invalid parameter value.
    #[must_use]
    pub fn invalid_parameter_value(message: impl Into<String>) -> Self {
        Self::new(SnsErrorCode::InvalidParameterValue, message)
    }

    /// Internal server error.
    #[must_use]
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(SnsErrorCode::InternalError, message)
    }

    /// Missing action parameter.
    #[must_use]
    pub fn missing_action() -> Self {
        Self::new(
            SnsErrorCode::MissingAction,
            "Missing required parameter: Action",
        )
    }

    /// Unknown operation.
    #[must_use]
    pub fn unknown_operation(name: &str) -> Self {
        Self::new(
            SnsErrorCode::InvalidParameterValue,
            format!("Unrecognized operation: {name}"),
        )
    }

    /// Invalid security token or credentials.
    #[must_use]
    pub fn invalid_security(message: impl Into<String>) -> Self {
        Self::new(SnsErrorCode::InvalidSecurity, message)
    }

    /// Authorization error.
    #[must_use]
    pub fn authorization_error(message: impl Into<String>) -> Self {
        Self::new(SnsErrorCode::AuthorizationError, message)
    }
}

/// Create an `SnsError` from an error code.
///
/// # Examples
///
/// ```
/// use rustack_sns_model::sns_error;
/// use rustack_sns_model::error::SnsErrorCode;
///
/// let err = sns_error!(NotFound, "Topic not found");
/// assert_eq!(err.code, SnsErrorCode::NotFound);
/// ```
#[macro_export]
macro_rules! sns_error {
    ($code:ident, $msg:expr) => {
        $crate::error::SnsError::new($crate::error::SnsErrorCode::$code, $msg)
    };
}
