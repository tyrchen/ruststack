//! Auto-generated from AWS SES Smithy model. DO NOT EDIT.
//!
//! SES errors use JSON format with a `__type` field containing the
//! short error type name (e.g., `ResourceNotFoundException`).

use std::fmt;

/// Well-known SES error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum SesErrorCode {
    /// AccountSendingPausedException error.
    #[default]
    AccountSendingPausedException,
    /// AlreadyExists error.
    AlreadyExists,
    /// AlreadyExistsException error.
    AlreadyExistsException,
    /// CannotDeleteException error.
    CannotDeleteException,
    /// ConfigurationSetAlreadyExistsException error.
    ConfigurationSetAlreadyExistsException,
    /// ConfigurationSetDoesNotExist error.
    ConfigurationSetDoesNotExist,
    /// ConfigurationSetDoesNotExistException error.
    ConfigurationSetDoesNotExistException,
    /// ConfigurationSetSendingPausedException error.
    ConfigurationSetSendingPausedException,
    /// EventDestinationAlreadyExistsException error.
    EventDestinationAlreadyExistsException,
    /// EventDestinationDoesNotExistException error.
    EventDestinationDoesNotExistException,
    /// InternalError error.
    InternalError,
    /// InvalidCloudWatchDestinationException error.
    InvalidCloudWatchDestinationException,
    /// InvalidConfigurationSetException error.
    InvalidConfigurationSetException,
    /// InvalidFirehoseDestinationException error.
    InvalidFirehoseDestinationException,
    /// InvalidLambdaFunctionException error.
    InvalidLambdaFunctionException,
    /// InvalidParameterValue error.
    InvalidParameterValue,
    /// InvalidPolicyException error.
    InvalidPolicyException,
    /// InvalidS3ConfigurationException error.
    InvalidS3ConfigurationException,
    /// InvalidSNSDestinationException error.
    InvalidSNSDestinationException,
    /// InvalidSnsTopicException error.
    InvalidSnsTopicException,
    /// InvalidTemplate error.
    InvalidTemplate,
    /// InvalidTemplateException error.
    InvalidTemplateException,
    /// LimitExceeded error.
    LimitExceeded,
    /// LimitExceededException error.
    LimitExceededException,
    /// MailFromDomainNotVerifiedException error.
    MailFromDomainNotVerifiedException,
    /// MessageRejected error.
    MessageRejected,
    /// MissingAction error.
    MissingAction,
    /// RuleDoesNotExist error.
    RuleDoesNotExist,
    /// RuleDoesNotExistException error.
    RuleDoesNotExistException,
    /// RuleSetDoesNotExist error.
    RuleSetDoesNotExist,
    /// RuleSetDoesNotExistException error.
    RuleSetDoesNotExistException,
    /// TemplateDoesNotExist error.
    TemplateDoesNotExist,
    /// TemplateDoesNotExistException error.
    TemplateDoesNotExistException,
}

impl SesErrorCode {
    /// Returns the short error type string for the JSON `__type` field.
    #[must_use]
    pub fn error_type(&self) -> &'static str {
        self.as_str()
    }

    /// Returns the short error code string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AccountSendingPausedException => "AccountSendingPausedException",
            Self::AlreadyExists => "AlreadyExists",
            Self::AlreadyExistsException => "AlreadyExistsException",
            Self::CannotDeleteException => "CannotDeleteException",
            Self::ConfigurationSetAlreadyExistsException => {
                "ConfigurationSetAlreadyExistsException"
            }
            Self::ConfigurationSetDoesNotExist => "ConfigurationSetDoesNotExist",
            Self::ConfigurationSetDoesNotExistException => "ConfigurationSetDoesNotExistException",
            Self::ConfigurationSetSendingPausedException => {
                "ConfigurationSetSendingPausedException"
            }
            Self::EventDestinationAlreadyExistsException => {
                "EventDestinationAlreadyExistsException"
            }
            Self::EventDestinationDoesNotExistException => "EventDestinationDoesNotExistException",
            Self::InternalError => "InternalError",
            Self::InvalidCloudWatchDestinationException => "InvalidCloudWatchDestinationException",
            Self::InvalidConfigurationSetException => "InvalidConfigurationSetException",
            Self::InvalidFirehoseDestinationException => "InvalidFirehoseDestinationException",
            Self::InvalidLambdaFunctionException => "InvalidLambdaFunctionException",
            Self::InvalidParameterValue => "InvalidParameterValue",
            Self::InvalidPolicyException => "InvalidPolicyException",
            Self::InvalidS3ConfigurationException => "InvalidS3ConfigurationException",
            Self::InvalidSNSDestinationException => "InvalidSNSDestinationException",
            Self::InvalidSnsTopicException => "InvalidSnsTopicException",
            Self::InvalidTemplate => "InvalidTemplate",
            Self::InvalidTemplateException => "InvalidTemplateException",
            Self::LimitExceeded => "LimitExceeded",
            Self::LimitExceededException => "LimitExceededException",
            Self::MailFromDomainNotVerifiedException => "MailFromDomainNotVerifiedException",
            Self::MessageRejected => "MessageRejected",
            Self::MissingAction => "MissingAction",
            Self::RuleDoesNotExist => "RuleDoesNotExist",
            Self::RuleDoesNotExistException => "RuleDoesNotExistException",
            Self::RuleSetDoesNotExist => "RuleSetDoesNotExist",
            Self::RuleSetDoesNotExistException => "RuleSetDoesNotExistException",
            Self::TemplateDoesNotExist => "TemplateDoesNotExist",
            Self::TemplateDoesNotExistException => "TemplateDoesNotExistException",
        }
    }

    /// The error code string used in the XML `<Code>` element.
    #[must_use]
    pub fn code(&self) -> &'static str {
        self.as_str()
    }

    /// HTTP status code for this error.
    #[must_use]
    pub fn status_code(&self) -> http::StatusCode {
        self.default_status_code()
    }

    /// Fault type: `"Sender"` for client errors, `"Receiver"` for server errors.
    #[must_use]
    pub fn fault(&self) -> &'static str {
        match self {
            Self::InternalError => "Receiver",
            _ => "Sender",
        }
    }

    /// Returns the default HTTP status code for this error.
    #[must_use]
    pub fn default_status_code(&self) -> http::StatusCode {
        match self {
            Self::AccountSendingPausedException
            | Self::AlreadyExists
            | Self::AlreadyExistsException
            | Self::CannotDeleteException
            | Self::ConfigurationSetAlreadyExistsException
            | Self::ConfigurationSetDoesNotExist
            | Self::ConfigurationSetDoesNotExistException
            | Self::ConfigurationSetSendingPausedException
            | Self::EventDestinationAlreadyExistsException
            | Self::EventDestinationDoesNotExistException
            | Self::InvalidCloudWatchDestinationException
            | Self::InvalidConfigurationSetException
            | Self::InvalidFirehoseDestinationException
            | Self::InvalidLambdaFunctionException
            | Self::InvalidParameterValue
            | Self::InvalidPolicyException
            | Self::InvalidS3ConfigurationException
            | Self::InvalidSNSDestinationException
            | Self::InvalidSnsTopicException
            | Self::InvalidTemplate
            | Self::InvalidTemplateException
            | Self::LimitExceeded
            | Self::LimitExceededException
            | Self::MailFromDomainNotVerifiedException
            | Self::MessageRejected
            | Self::MissingAction
            | Self::RuleDoesNotExist
            | Self::RuleDoesNotExistException
            | Self::RuleSetDoesNotExist
            | Self::RuleSetDoesNotExistException
            | Self::TemplateDoesNotExist
            | Self::TemplateDoesNotExistException => http::StatusCode::BAD_REQUEST,
            Self::InternalError => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl fmt::Display for SesErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An SES error response.
#[derive(Debug)]
pub struct SesError {
    /// The error code.
    pub code: SesErrorCode,
    /// A human-readable error message.
    pub message: String,
    /// The HTTP status code.
    pub status_code: http::StatusCode,
    /// The underlying source error, if any.
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for SesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SesError({}): {}", self.code, self.message)
    }
}

impl std::error::Error for SesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl SesError {
    /// Create a new `SesError` from an error code.
    #[must_use]
    pub fn new(code: SesErrorCode) -> Self {
        Self {
            status_code: code.default_status_code(),
            message: code.as_str().to_owned(),
            code,
            source: None,
        }
    }

    /// Create a new `SesError` with a custom message.
    #[must_use]
    pub fn with_message(code: SesErrorCode, message: impl Into<String>) -> Self {
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
        Self::with_message(SesErrorCode::InternalError, message)
    }

    /// Missing action header.
    #[must_use]
    pub fn missing_action() -> Self {
        Self::with_message(
            SesErrorCode::MissingAction,
            "Missing required header: X-Amz-Target",
        )
    }

    /// Not implemented.
    #[must_use]
    pub fn not_implemented(operation: &str) -> Self {
        Self::with_message(
            SesErrorCode::InternalError,
            format!("Operation {operation} is not yet implemented"),
        )
    }

    /// Invalid parameter value.
    #[must_use]
    pub fn invalid_parameter_value(message: impl Into<String>) -> Self {
        Self::with_message(SesErrorCode::InvalidParameterValue, message)
    }

    /// Unknown/invalid action.
    #[must_use]
    pub fn invalid_action(action: &str) -> Self {
        Self::with_message(
            SesErrorCode::InvalidParameterValue,
            format!("Unrecognized operation: {action}"),
        )
    }

    /// Message rejected.
    #[must_use]
    pub fn message_rejected(message: impl Into<String>) -> Self {
        Self::with_message(SesErrorCode::MessageRejected, message)
    }

    /// Template does not exist.
    #[must_use]
    pub fn template_does_not_exist(name: &str) -> Self {
        Self::with_message(
            SesErrorCode::TemplateDoesNotExist,
            format!("Template {name} does not exist"),
        )
    }
}

/// Create an `SesError` from an error code.
///
/// # Examples
///
/// ```ignore
/// let err = ses_error!(AccountSendingPausedException);
/// assert_eq!(err.code, SesErrorCode::AccountSendingPausedException);
/// ```
#[macro_export]
macro_rules! ses_error {
    ($code:ident) => {
        $crate::error::SesError::new($crate::error::SesErrorCode::$code)
    };
    ($code:ident, $msg:expr) => {
        $crate::error::SesError::with_message($crate::error::SesErrorCode::$code, $msg)
    };
}
