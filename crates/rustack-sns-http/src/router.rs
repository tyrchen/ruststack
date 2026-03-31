//! SNS request router.
//!
//! SNS uses the `awsQuery` protocol where requests are `POST /` with
//! `Content-Type: application/x-www-form-urlencoded`. The operation is
//! specified by the `Action=<OperationName>` form parameter.

use rustack_sns_model::{error::SnsError, operations::SnsOperation};

/// Resolve an SNS operation from parsed form parameters.
///
/// Looks for the `Action` parameter in the form body and maps it to
/// the corresponding [`SnsOperation`] variant.
pub fn resolve_operation(params: &[(String, String)]) -> Result<SnsOperation, SnsError> {
    let action = params
        .iter()
        .find(|(k, _)| k == "Action")
        .map(|(_, v)| v.as_str())
        .ok_or_else(SnsError::missing_action)?;

    SnsOperation::from_name(action).ok_or_else(|| SnsError::unknown_operation(action))
}

#[cfg(test)]
mod tests {
    use rustack_sns_model::error::SnsErrorCode;

    use super::*;

    fn params_with_action(action: &str) -> Vec<(String, String)> {
        vec![("Action".to_owned(), action.to_owned())]
    }

    #[test]
    fn test_should_resolve_create_topic() {
        let params = params_with_action("CreateTopic");
        let op = resolve_operation(&params).unwrap();
        assert_eq!(op, SnsOperation::CreateTopic);
    }

    #[test]
    fn test_should_resolve_publish() {
        let params = params_with_action("Publish");
        let op = resolve_operation(&params).unwrap();
        assert_eq!(op, SnsOperation::Publish);
    }

    #[test]
    fn test_should_resolve_subscribe() {
        let params = params_with_action("Subscribe");
        let op = resolve_operation(&params).unwrap();
        assert_eq!(op, SnsOperation::Subscribe);
    }

    #[test]
    fn test_should_error_on_missing_action() {
        let params: Vec<(String, String)> = vec![];
        let err = resolve_operation(&params).unwrap_err();
        assert_eq!(err.code, SnsErrorCode::MissingAction);
    }

    #[test]
    fn test_should_error_on_unknown_operation() {
        let params = params_with_action("NonExistentOperation");
        let err = resolve_operation(&params).unwrap_err();
        assert_eq!(err.code, SnsErrorCode::InvalidParameterValue);
        assert!(err.message.contains("NonExistentOperation"));
    }

    #[test]
    fn test_should_find_action_among_other_params() {
        let params = vec![
            ("Version".to_owned(), "2010-03-31".to_owned()),
            ("Action".to_owned(), "DeleteTopic".to_owned()),
            (
                "TopicArn".to_owned(),
                "arn:aws:sns:us-east-1:123456789012:MyTopic".to_owned(),
            ),
        ];
        let op = resolve_operation(&params).unwrap();
        assert_eq!(op, SnsOperation::DeleteTopic);
    }
}
