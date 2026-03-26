//! IAM request router.
//!
//! IAM uses the `awsQuery` protocol where requests are `POST /` with
//! `Content-Type: application/x-www-form-urlencoded`. The operation is
//! specified by the `Action=<OperationName>` form parameter.

use ruststack_iam_model::{error::IamError, operations::IamOperation};

/// Resolve an IAM operation from parsed form parameters.
///
/// Looks for the `Action` parameter in the form body and maps it to
/// the corresponding [`IamOperation`] variant.
pub fn resolve_operation(params: &[(String, String)]) -> Result<IamOperation, IamError> {
    let action = params
        .iter()
        .find(|(k, _)| k == "Action")
        .map(|(_, v)| v.as_str())
        .ok_or_else(IamError::missing_action)?;

    IamOperation::from_name(action).ok_or_else(|| IamError::unknown_operation(action))
}

#[cfg(test)]
mod tests {
    use ruststack_iam_model::error::IamErrorCode;

    use super::*;

    fn params_with_action(action: &str) -> Vec<(String, String)> {
        vec![("Action".to_owned(), action.to_owned())]
    }

    #[test]
    fn test_should_resolve_create_user() {
        let params = params_with_action("CreateUser");
        let op = resolve_operation(&params).unwrap();
        assert_eq!(op, IamOperation::CreateUser);
    }

    #[test]
    fn test_should_resolve_get_role() {
        let params = params_with_action("GetRole");
        let op = resolve_operation(&params).unwrap();
        assert_eq!(op, IamOperation::GetRole);
    }

    #[test]
    fn test_should_resolve_list_policies() {
        let params = params_with_action("ListPolicies");
        let op = resolve_operation(&params).unwrap();
        assert_eq!(op, IamOperation::ListPolicies);
    }

    #[test]
    fn test_should_error_on_missing_action() {
        let params: Vec<(String, String)> = vec![];
        let err = resolve_operation(&params).unwrap_err();
        assert_eq!(err.code, IamErrorCode::MissingAction);
    }

    #[test]
    fn test_should_error_on_unknown_operation() {
        let params = params_with_action("NonExistentOperation");
        let err = resolve_operation(&params).unwrap_err();
        assert_eq!(err.code, IamErrorCode::InvalidAction);
        assert!(err.message.contains("NonExistentOperation"));
    }

    #[test]
    fn test_should_find_action_among_other_params() {
        let params = vec![
            ("Version".to_owned(), "2010-05-08".to_owned()),
            ("Action".to_owned(), "DeleteUser".to_owned()),
            ("UserName".to_owned(), "testuser".to_owned()),
        ];
        let op = resolve_operation(&params).unwrap();
        assert_eq!(op, IamOperation::DeleteUser);
    }
}
