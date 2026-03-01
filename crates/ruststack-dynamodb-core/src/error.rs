//! Core error type for DynamoDB operations.

use ruststack_dynamodb_model::error::{DynamoDBError, DynamoDBErrorCode};

/// Convert a storage error into a DynamoDB validation error.
///
/// Takes `e` by value because this is used as a closure argument to `.map_err()`.
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn storage_error_to_dynamodb(e: crate::storage::StorageError) -> DynamoDBError {
    DynamoDBError::with_message(DynamoDBErrorCode::ValidationException, e.to_string())
}

/// Convert an expression error into a DynamoDB validation error.
///
/// Takes `e` by value because this is used as a closure argument to `.map_err()`.
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn expression_error_to_dynamodb(e: crate::expression::ExpressionError) -> DynamoDBError {
    DynamoDBError::with_message(DynamoDBErrorCode::ValidationException, e.to_string())
}
