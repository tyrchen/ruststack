//! SQS response serialization and error formatting.

use ruststack_sqs_model::error::SqsError;

use crate::body::SqsResponseBody;

/// Content type for SQS JSON responses.
pub const CONTENT_TYPE: &str = "application/x-amz-json-1.0";

/// Serialize an SQS error into a JSON response body.
///
/// The error format follows the AWS SQS JSON protocol:
///
/// ```json
/// {
///   "__type": "AWS.SimpleQueueService.NonExistentQueue",
///   "message": "The specified queue does not exist."
/// }
/// ```
#[must_use]
pub fn error_to_json(error: &SqsError) -> Vec<u8> {
    let obj = serde_json::json!({
        "__type": error.code.error_type(),
        "message": error.message,
    });
    serde_json::to_vec(&obj).expect("JSON serialization of error cannot fail")
}

/// Convert an `SqsError` into a complete HTTP error response.
///
/// Includes the `x-amzn-query-error` header for `awsQueryCompatible` support.
#[must_use]
pub fn error_to_response(error: &SqsError, request_id: &str) -> http::Response<SqsResponseBody> {
    let json = error_to_json(error);
    let crc = crc32fast::hash(&json);
    let body = SqsResponseBody::from_json(json);

    let mut response = http::Response::builder()
        .status(error.code.status_code())
        .header("content-type", CONTENT_TYPE)
        .header("x-amzn-requestid", request_id)
        .header("x-amzn-query-error", error.code.query_error_header())
        .body(body)
        .expect("valid error response");

    if let Ok(hv) = http::HeaderValue::from_str(&crc.to_string()) {
        response.headers_mut().insert("x-amz-crc32", hv);
    }

    response
}

/// Build a success response from JSON bytes.
#[must_use]
pub fn json_response(json: Vec<u8>, request_id: &str) -> http::Response<SqsResponseBody> {
    let crc = crc32fast::hash(&json);
    let body = SqsResponseBody::from_json(json);

    let mut response = http::Response::builder()
        .status(http::StatusCode::OK)
        .header("content-type", CONTENT_TYPE)
        .header("x-amzn-requestid", request_id)
        .body(body)
        .expect("valid JSON response");

    if let Ok(hv) = http::HeaderValue::from_str(&crc.to_string()) {
        response.headers_mut().insert("x-amz-crc32", hv);
    }

    response
}

#[cfg(test)]
mod tests {
    use ruststack_sqs_model::error::SqsErrorCode;

    use super::*;

    #[test]
    fn test_should_format_error_json() {
        let err = SqsError::new(
            SqsErrorCode::NonExistentQueue,
            "The specified queue does not exist.",
        );
        let json = error_to_json(&err);
        let parsed: serde_json::Value = serde_json::from_slice(&json).unwrap();
        assert_eq!(parsed["__type"], "AWS.SimpleQueueService.NonExistentQueue");
        assert_eq!(parsed["message"], "The specified queue does not exist.");
    }

    #[test]
    fn test_should_build_error_response_with_query_error_header() {
        let err = SqsError::new(
            SqsErrorCode::NonExistentQueue,
            "The specified queue does not exist.",
        );
        let resp = error_to_response(&err, "test-req-123");
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
        assert_eq!(resp.headers().get("content-type").unwrap(), CONTENT_TYPE);
        assert_eq!(
            resp.headers().get("x-amzn-query-error").unwrap(),
            "AWS.SimpleQueueService.NonExistentQueue;Sender",
        );
        assert_eq!(
            resp.headers().get("x-amzn-requestid").unwrap(),
            "test-req-123",
        );
        assert!(resp.headers().get("x-amz-crc32").is_some());
    }

    #[test]
    fn test_should_build_json_success_response() {
        let json = serde_json::to_vec(
            &serde_json::json!({"QueueUrl": "http://localhost:4566/000000000000/test"}),
        )
        .unwrap();
        let resp = json_response(json, "req-456");
        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!(resp.headers().get("content-type").unwrap(), CONTENT_TYPE);
        assert!(resp.headers().get("x-amz-crc32").is_some());
    }
}
