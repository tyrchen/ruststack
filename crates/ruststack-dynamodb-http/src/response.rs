//! DynamoDB response serialization and error formatting.

use ruststack_dynamodb_model::error::DynamoDBError;

use crate::body::DynamoDBResponseBody;

/// Content type for DynamoDB JSON responses.
pub const CONTENT_TYPE: &str = "application/x-amz-json-1.0";

/// Serialize a DynamoDB error into a JSON response body.
///
/// The error format follows the AWS DynamoDB JSON protocol:
///
/// ```json
/// {
///   "__type": "com.amazonaws.dynamodb.v20120810#ResourceNotFoundException",
///   "Message": "Requested resource not found"
/// }
/// ```
#[must_use]
pub fn error_to_json(error: &DynamoDBError) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "__type": error.error_type(),
        "Message": error.message,
    }))
    .expect("JSON serialization of error cannot fail")
}

/// Convert a `DynamoDBError` into a complete HTTP error response.
#[must_use]
pub fn error_to_response(
    error: &DynamoDBError,
    request_id: &str,
) -> http::Response<DynamoDBResponseBody> {
    let json = error_to_json(error);
    let crc = crc32fast::hash(&json);
    let body = DynamoDBResponseBody::from_json(json);

    let mut response = http::Response::builder()
        .status(error.status_code)
        .header("content-type", CONTENT_TYPE)
        .header("x-amzn-requestid", request_id)
        .body(body)
        .expect("valid error response");

    if let Ok(hv) = http::HeaderValue::from_str(&crc.to_string()) {
        response.headers_mut().insert("x-amz-crc32", hv);
    }

    response
}

/// Build a success response from JSON bytes.
#[must_use]
pub fn json_response(json: Vec<u8>, request_id: &str) -> http::Response<DynamoDBResponseBody> {
    let crc = crc32fast::hash(&json);
    let body = DynamoDBResponseBody::from_json(json);

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
    use super::*;
    use ruststack_dynamodb_model::error::DynamoDBErrorCode;

    #[test]
    fn test_should_format_error_json() {
        let err = DynamoDBError::with_message(
            DynamoDBErrorCode::ResourceNotFoundException,
            "Table 'users' not found",
        );
        let json = error_to_json(&err);
        let parsed: serde_json::Value = serde_json::from_slice(&json).unwrap();
        assert_eq!(
            parsed["__type"],
            "com.amazonaws.dynamodb.v20120810#ResourceNotFoundException"
        );
        assert_eq!(parsed["Message"], "Table 'users' not found");
    }

    #[test]
    fn test_should_build_error_response_with_correct_status() {
        let err = DynamoDBError::with_message(
            DynamoDBErrorCode::ValidationException,
            "Missing required key",
        );
        let resp = error_to_response(&err, "test-req-123");
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
        assert_eq!(resp.headers().get("content-type").unwrap(), CONTENT_TYPE,);
        assert_eq!(
            resp.headers().get("x-amzn-requestid").unwrap(),
            "test-req-123",
        );
        assert!(resp.headers().get("x-amz-crc32").is_some());
    }

    #[test]
    fn test_should_build_json_success_response() {
        let json = serde_json::to_vec(&serde_json::json!({"TableNames": ["users"]})).unwrap();
        let resp = json_response(json, "req-456");
        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!(resp.headers().get("content-type").unwrap(), CONTENT_TYPE,);
        assert!(resp.headers().get("x-amz-crc32").is_some());
    }
}
