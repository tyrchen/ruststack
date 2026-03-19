//! SES v1 request router.
//!
//! SES v1 uses the `awsQuery` protocol where requests are `POST /` with
//! `Content-Type: application/x-www-form-urlencoded`. The operation is
//! specified by the `Action=<OperationName>` form parameter.

use ruststack_ses_model::error::SesError;
use ruststack_ses_model::operations::SesOperation;

/// Resolve an SES v1 action string to an `SesOperation`.
///
/// # Errors
///
/// Returns `SesError` if the action is not recognized.
pub fn resolve_action(action: &str) -> Result<SesOperation, SesError> {
    SesOperation::from_name(action).ok_or_else(|| SesError::invalid_action(action))
}

/// Resolve an SES operation from parsed form parameters.
///
/// Looks for the `Action` parameter in the form body and maps it to
/// the corresponding [`SesOperation`] variant.
pub fn resolve_operation(params: &[(String, String)]) -> Result<SesOperation, SesError> {
    let action = params
        .iter()
        .find(|(k, _)| k == "Action")
        .map(|(_, v)| v.as_str())
        .ok_or_else(SesError::missing_action)?;

    resolve_action(action)
}

#[cfg(test)]
mod tests {
    use ruststack_ses_model::error::SesErrorCode;

    use super::*;

    fn params_with_action(action: &str) -> Vec<(String, String)> {
        vec![("Action".to_owned(), action.to_owned())]
    }

    #[test]
    fn test_should_resolve_send_email() {
        let params = params_with_action("SendEmail");
        let op = resolve_operation(&params).unwrap();
        assert_eq!(op, SesOperation::SendEmail);
    }

    #[test]
    fn test_should_resolve_verify_email_identity() {
        let params = params_with_action("VerifyEmailIdentity");
        let op = resolve_operation(&params).unwrap();
        assert_eq!(op, SesOperation::VerifyEmailIdentity);
    }

    #[test]
    fn test_should_error_on_missing_action() {
        let params: Vec<(String, String)> = vec![];
        let err = resolve_operation(&params).unwrap_err();
        assert_eq!(err.code, SesErrorCode::MissingAction);
    }

    #[test]
    fn test_should_error_on_unknown_operation() {
        let params = params_with_action("NonExistentOperation");
        let err = resolve_operation(&params).unwrap_err();
        assert_eq!(err.code, SesErrorCode::InvalidParameterValue);
        assert!(err.message.contains("NonExistentOperation"));
    }

    #[test]
    fn test_should_find_action_among_other_params() {
        let params = vec![
            ("Version".to_owned(), "2010-12-01".to_owned()),
            ("Action".to_owned(), "ListIdentities".to_owned()),
            ("IdentityType".to_owned(), "EmailAddress".to_owned()),
        ];
        let op = resolve_operation(&params).unwrap();
        assert_eq!(op, SesOperation::ListIdentities);
    }
}
