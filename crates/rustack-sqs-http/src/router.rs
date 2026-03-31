//! SQS request router.
//!
//! SQS uses the `awsJson1_0` protocol where all requests are `POST /`
//! with the operation specified in the `X-Amz-Target` header:
//!
//! ```text
//! X-Amz-Target: AmazonSQS.CreateQueue
//! ```

use rustack_sqs_model::{error::SqsError, operations::SqsOperation};

/// The expected prefix for the `X-Amz-Target` header value.
const TARGET_PREFIX: &str = "AmazonSQS.";

/// Resolve an SQS operation from HTTP request headers.
///
/// Extracts the operation from the `X-Amz-Target` header, validates the
/// format, and maps it to an [`SqsOperation`] enum variant.
pub fn resolve_operation(headers: &http::HeaderMap) -> Result<SqsOperation, SqsError> {
    let target = headers
        .get("x-amz-target")
        .ok_or_else(SqsError::missing_action)?;

    let target_str = target.to_str().map_err(|_| SqsError::missing_action())?;

    let operation_name = target_str
        .strip_prefix(TARGET_PREFIX)
        .ok_or_else(|| SqsError::unknown_operation(target_str))?;

    SqsOperation::from_name(operation_name).ok_or_else(|| SqsError::unknown_operation(target_str))
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
    fn test_should_resolve_create_queue() {
        let headers = headers_with_target("AmazonSQS.CreateQueue");
        let op = resolve_operation(&headers).unwrap();
        assert_eq!(op, SqsOperation::CreateQueue);
    }

    #[test]
    fn test_should_resolve_all_operations() {
        let ops = [
            ("AmazonSQS.CreateQueue", SqsOperation::CreateQueue),
            ("AmazonSQS.DeleteQueue", SqsOperation::DeleteQueue),
            ("AmazonSQS.GetQueueUrl", SqsOperation::GetQueueUrl),
            ("AmazonSQS.ListQueues", SqsOperation::ListQueues),
            ("AmazonSQS.SendMessage", SqsOperation::SendMessage),
            ("AmazonSQS.ReceiveMessage", SqsOperation::ReceiveMessage),
            ("AmazonSQS.DeleteMessage", SqsOperation::DeleteMessage),
            (
                "AmazonSQS.GetQueueAttributes",
                SqsOperation::GetQueueAttributes,
            ),
            (
                "AmazonSQS.SetQueueAttributes",
                SqsOperation::SetQueueAttributes,
            ),
            ("AmazonSQS.PurgeQueue", SqsOperation::PurgeQueue),
            ("AmazonSQS.SendMessageBatch", SqsOperation::SendMessageBatch),
            (
                "AmazonSQS.DeleteMessageBatch",
                SqsOperation::DeleteMessageBatch,
            ),
            (
                "AmazonSQS.ChangeMessageVisibility",
                SqsOperation::ChangeMessageVisibility,
            ),
            (
                "AmazonSQS.ChangeMessageVisibilityBatch",
                SqsOperation::ChangeMessageVisibilityBatch,
            ),
            ("AmazonSQS.TagQueue", SqsOperation::TagQueue),
            ("AmazonSQS.UntagQueue", SqsOperation::UntagQueue),
            ("AmazonSQS.ListQueueTags", SqsOperation::ListQueueTags),
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
            rustack_sqs_model::error::SqsErrorCode::MissingAction,
        );
    }

    #[test]
    fn test_should_error_on_wrong_prefix() {
        let headers = headers_with_target("DynamoDB_20120810.CreateTable");
        let err = resolve_operation(&headers).unwrap_err();
        assert_eq!(
            err.code,
            rustack_sqs_model::error::SqsErrorCode::InvalidParameterValue,
        );
    }

    #[test]
    fn test_should_error_on_unknown_operation() {
        let headers = headers_with_target("AmazonSQS.NonExistent");
        let err = resolve_operation(&headers).unwrap_err();
        assert_eq!(
            err.code,
            rustack_sqs_model::error::SqsErrorCode::InvalidParameterValue,
        );
    }
}
