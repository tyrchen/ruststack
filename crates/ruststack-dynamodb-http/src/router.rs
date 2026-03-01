//! DynamoDB request router.
//!
//! DynamoDB uses the `awsJson1_0` protocol where all requests are `POST /`
//! with the operation specified in the `X-Amz-Target` header:
//!
//! ```text
//! X-Amz-Target: DynamoDB_20120810.CreateTable
//! ```
//!
//! This makes routing ~20 lines compared to S3's ~400 lines of path/query
//! parsing and virtual-host resolution.

use ruststack_dynamodb_model::error::DynamoDBError;
use ruststack_dynamodb_model::operations::DynamoDBOperation;

/// The expected prefix for the `X-Amz-Target` header value.
const TARGET_PREFIX: &str = "DynamoDB_20120810.";

/// Resolve a DynamoDB operation from an HTTP request.
///
/// Extracts the operation from the `X-Amz-Target` header, validates the
/// format, and maps it to a [`DynamoDBOperation`] enum variant.
pub fn resolve_operation(headers: &http::HeaderMap) -> Result<DynamoDBOperation, DynamoDBError> {
    let target = headers
        .get("x-amz-target")
        .ok_or_else(DynamoDBError::missing_action)?;

    let target_str = target
        .to_str()
        .map_err(|_| DynamoDBError::missing_action())?;

    let operation_name = target_str
        .strip_prefix(TARGET_PREFIX)
        .ok_or_else(|| DynamoDBError::unknown_operation(target_str))?;

    DynamoDBOperation::from_name(operation_name)
        .ok_or_else(|| DynamoDBError::unknown_operation(target_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers_with_target(target: &str) -> http::HeaderMap {
        let mut map = http::HeaderMap::new();
        map.insert("x-amz-target", http::HeaderValue::from_str(target).unwrap());
        map
    }

    #[test]
    fn test_should_resolve_create_table() {
        let headers = headers_with_target("DynamoDB_20120810.CreateTable");
        let op = resolve_operation(&headers).unwrap();
        assert_eq!(op, DynamoDBOperation::CreateTable);
    }

    #[test]
    fn test_should_resolve_all_operations() {
        let ops = [
            (
                "DynamoDB_20120810.CreateTable",
                DynamoDBOperation::CreateTable,
            ),
            (
                "DynamoDB_20120810.DeleteTable",
                DynamoDBOperation::DeleteTable,
            ),
            (
                "DynamoDB_20120810.DescribeTable",
                DynamoDBOperation::DescribeTable,
            ),
            (
                "DynamoDB_20120810.ListTables",
                DynamoDBOperation::ListTables,
            ),
            ("DynamoDB_20120810.PutItem", DynamoDBOperation::PutItem),
            ("DynamoDB_20120810.GetItem", DynamoDBOperation::GetItem),
            (
                "DynamoDB_20120810.UpdateItem",
                DynamoDBOperation::UpdateItem,
            ),
            (
                "DynamoDB_20120810.DeleteItem",
                DynamoDBOperation::DeleteItem,
            ),
            ("DynamoDB_20120810.Query", DynamoDBOperation::Query),
            ("DynamoDB_20120810.Scan", DynamoDBOperation::Scan),
            (
                "DynamoDB_20120810.BatchGetItem",
                DynamoDBOperation::BatchGetItem,
            ),
            (
                "DynamoDB_20120810.BatchWriteItem",
                DynamoDBOperation::BatchWriteItem,
            ),
        ];
        for (target, expected) in ops {
            let headers = headers_with_target(target);
            let op = resolve_operation(&headers).unwrap();
            assert_eq!(op, expected, "failed for target: {target}");
        }
    }

    #[test]
    fn test_should_error_on_missing_target() {
        let headers = http::HeaderMap::new();
        let err = resolve_operation(&headers).unwrap_err();
        assert_eq!(
            err.code,
            ruststack_dynamodb_model::error::DynamoDBErrorCode::MissingAction,
        );
    }

    #[test]
    fn test_should_error_on_wrong_prefix() {
        let headers = headers_with_target("WrongService.CreateTable");
        let err = resolve_operation(&headers).unwrap_err();
        assert_eq!(
            err.code,
            ruststack_dynamodb_model::error::DynamoDBErrorCode::UnrecognizedClientException,
        );
    }

    #[test]
    fn test_should_error_on_unknown_operation() {
        let headers = headers_with_target("DynamoDB_20120810.NonExistent");
        let err = resolve_operation(&headers).unwrap_err();
        assert_eq!(
            err.code,
            ruststack_dynamodb_model::error::DynamoDBErrorCode::UnrecognizedClientException,
        );
    }
}
