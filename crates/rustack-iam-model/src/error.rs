//! IAM error types for the awsQuery XML protocol.
//!
//! IAM errors use XML format with `<Error>` elements containing
//! `<Type>`, `<Code>`, and `<Message>` fields.

use std::fmt;

/// Well-known IAM error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum IamErrorCode {
    /// ConcurrentModificationException error.
    #[default]
    ConcurrentModificationException,
    /// DeleteConflict error.
    DeleteConflict,
    /// DeleteConflictException error.
    DeleteConflictException,
    /// EntityAlreadyExists error.
    EntityAlreadyExists,
    /// EntityAlreadyExistsException error.
    EntityAlreadyExistsException,
    /// EntityTemporarilyUnmodifiableException error.
    EntityTemporarilyUnmodifiableException,
    /// InvalidAction error.
    InvalidAction,
    /// InvalidInput error.
    InvalidInput,
    /// InvalidInputException error.
    InvalidInputException,
    /// LimitExceeded error.
    LimitExceeded,
    /// LimitExceededException error.
    LimitExceededException,
    /// MalformedPolicyDocument error.
    MalformedPolicyDocument,
    /// MalformedPolicyDocumentException error.
    MalformedPolicyDocumentException,
    /// MissingAction error.
    MissingAction,
    /// NoSuchEntity error.
    NoSuchEntity,
    /// NoSuchEntityException error.
    NoSuchEntityException,
    /// PolicyEvaluationException error.
    PolicyEvaluationException,
    /// PolicyNotAttachableException error.
    PolicyNotAttachableException,
    /// ServiceFailure error.
    ServiceFailure,
    /// ServiceFailureException error.
    ServiceFailureException,
    /// UnmodifiableEntityException error.
    UnmodifiableEntityException,
}

impl IamErrorCode {
    /// Returns the short error type string for the JSON `__type` field.
    #[must_use]
    pub fn error_type(&self) -> &'static str {
        self.as_str()
    }

    /// Returns the short error code string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConcurrentModificationException => "ConcurrentModificationException",
            Self::DeleteConflict => "DeleteConflict",
            Self::DeleteConflictException => "DeleteConflictException",
            Self::EntityAlreadyExists => "EntityAlreadyExists",
            Self::EntityAlreadyExistsException => "EntityAlreadyExistsException",
            Self::EntityTemporarilyUnmodifiableException => {
                "EntityTemporarilyUnmodifiableException"
            }
            Self::InvalidAction => "InvalidAction",
            Self::InvalidInput => "InvalidInput",
            Self::InvalidInputException => "InvalidInputException",
            Self::LimitExceeded => "LimitExceeded",
            Self::LimitExceededException => "LimitExceededException",
            Self::MalformedPolicyDocument => "MalformedPolicyDocument",
            Self::MalformedPolicyDocumentException => "MalformedPolicyDocumentException",
            Self::MissingAction => "MissingAction",
            Self::NoSuchEntity => "NoSuchEntity",
            Self::NoSuchEntityException => "NoSuchEntityException",
            Self::PolicyEvaluationException => "PolicyEvaluationException",
            Self::PolicyNotAttachableException => "PolicyNotAttachableException",
            Self::ServiceFailure => "ServiceFailure",
            Self::ServiceFailureException => "ServiceFailureException",
            Self::UnmodifiableEntityException => "UnmodifiableEntityException",
        }
    }

    /// Returns the error code string for the XML `<Code>` element.
    #[must_use]
    pub fn code(&self) -> &'static str {
        self.as_str()
    }

    /// Returns whether this is a `Sender` or `Receiver` fault for XML `<Type>`.
    #[must_use]
    pub fn fault(&self) -> &'static str {
        match self {
            Self::ServiceFailure
            | Self::ServiceFailureException
            | Self::PolicyEvaluationException => "Receiver",
            _ => "Sender",
        }
    }

    /// HTTP status code for this error.
    #[must_use]
    pub fn status_code(&self) -> http::StatusCode {
        self.default_status_code()
    }

    /// Returns the default HTTP status code for this error.
    #[must_use]
    pub fn default_status_code(&self) -> http::StatusCode {
        match self {
            Self::InvalidAction
            | Self::InvalidInput
            | Self::InvalidInputException
            | Self::MalformedPolicyDocument
            | Self::MalformedPolicyDocumentException
            | Self::MissingAction
            | Self::PolicyNotAttachableException
            | Self::UnmodifiableEntityException => http::StatusCode::BAD_REQUEST,
            Self::NoSuchEntity | Self::NoSuchEntityException => http::StatusCode::NOT_FOUND,
            Self::ConcurrentModificationException
            | Self::DeleteConflict
            | Self::DeleteConflictException
            | Self::EntityAlreadyExists
            | Self::EntityAlreadyExistsException
            | Self::EntityTemporarilyUnmodifiableException
            | Self::LimitExceeded
            | Self::LimitExceededException => http::StatusCode::CONFLICT,
            Self::PolicyEvaluationException
            | Self::ServiceFailure
            | Self::ServiceFailureException => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl fmt::Display for IamErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An IAM error response.
#[derive(Debug)]
pub struct IamError {
    /// The error code.
    pub code: IamErrorCode,
    /// A human-readable error message.
    pub message: String,
    /// The HTTP status code.
    pub status_code: http::StatusCode,
    /// The underlying source error, if any.
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for IamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IamError({}): {}", self.code, self.message)
    }
}

impl std::error::Error for IamError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl IamError {
    /// Create a new `IamError` from an error code.
    #[must_use]
    pub fn new(code: IamErrorCode) -> Self {
        Self {
            status_code: code.default_status_code(),
            message: code.as_str().to_owned(),
            code,
            source: None,
        }
    }

    /// Create a new `IamError` with a custom message.
    #[must_use]
    pub fn with_message(code: IamErrorCode, message: impl Into<String>) -> Self {
        Self {
            status_code: code.default_status_code(),
            message: message.into(),
            code,
            source: None,
        }
    }

    /// Returns the `__type` string for the JSON error response.
    #[must_use]
    pub fn error_type(&self) -> &'static str {
        self.code.error_type()
    }

    /// Internal error.
    #[must_use]
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::with_message(IamErrorCode::ServiceFailure, message)
    }

    /// Missing action parameter.
    #[must_use]
    pub fn missing_action() -> Self {
        Self::with_message(IamErrorCode::MissingAction, "Missing Action parameter")
    }

    /// Unknown operation.
    #[must_use]
    pub fn unknown_operation(action: &str) -> Self {
        Self::with_message(
            IamErrorCode::InvalidAction,
            format!("The action {action} is not valid for this endpoint."),
        )
    }

    /// Not implemented.
    #[must_use]
    pub fn not_implemented(operation: &str) -> Self {
        Self::with_message(
            IamErrorCode::ServiceFailure,
            format!("Operation {operation} is not yet implemented"),
        )
    }

    /// Entity not found.
    #[must_use]
    pub fn no_such_entity(message: impl Into<String>) -> Self {
        Self::with_message(IamErrorCode::NoSuchEntity, message)
    }

    /// Entity already exists.
    #[must_use]
    pub fn entity_already_exists(message: impl Into<String>) -> Self {
        Self::with_message(IamErrorCode::EntityAlreadyExists, message)
    }

    /// Delete conflict (entity has subordinate entities).
    #[must_use]
    pub fn delete_conflict(message: impl Into<String>) -> Self {
        Self::with_message(IamErrorCode::DeleteConflict, message)
    }

    /// Limit exceeded.
    #[must_use]
    pub fn limit_exceeded(message: impl Into<String>) -> Self {
        Self::with_message(IamErrorCode::LimitExceeded, message)
    }

    /// Malformed policy document.
    #[must_use]
    pub fn malformed_policy_document(message: impl Into<String>) -> Self {
        Self::with_message(IamErrorCode::MalformedPolicyDocument, message)
    }

    /// Invalid input.
    #[must_use]
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::with_message(IamErrorCode::InvalidInput, message)
    }

    /// Invalid security / auth error.
    #[must_use]
    pub fn invalid_security(message: impl Into<String>) -> Self {
        Self::with_message(IamErrorCode::InvalidInput, message)
    }
}

/// Create an `IamError` from an error code.
///
/// # Examples
///
/// ```ignore
/// let err = iam_error!(ConcurrentModificationException);
/// assert_eq!(err.code, IamErrorCode::ConcurrentModificationException);
/// ```
#[macro_export]
macro_rules! iam_error {
    ($code:ident) => {
        $crate::error::IamError::new($crate::error::IamErrorCode::$code)
    };
    ($code:ident, $msg:expr) => {
        $crate::error::IamError::with_message($crate::error::IamErrorCode::$code, $msg)
    };
}
