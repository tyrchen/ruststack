//! Gateway service that routes requests to S3 or DynamoDB.
//!
//! The gateway inspects the `X-Amz-Target` header to determine whether a
//! request is destined for DynamoDB (target starts with `DynamoDB_`) or
//! should fall through to the S3 service (the default).
//!
//! Health-check endpoints (`/_localstack/health`, `/_health`, `/health`) are
//! intercepted at the gateway level and return a combined status for all services.

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;

use http_body_util::Either;
use hyper::body::Incoming;
use hyper::service::Service;

use ruststack_dynamodb_http::body::DynamoDBResponseBody;
use ruststack_dynamodb_http::dispatch::DynamoDBHandler;
use ruststack_dynamodb_http::service::DynamoDBHttpService;
use ruststack_s3_http::body::S3ResponseBody;
use ruststack_s3_http::dispatch::S3Handler;
use ruststack_s3_http::service::S3HttpService;

/// Unified response body type combining both services.
pub type GatewayBody = Either<S3ResponseBody, DynamoDBResponseBody>;

/// Gateway that routes incoming HTTP requests to either the S3 or DynamoDB service.
///
/// Routing is based on the `X-Amz-Target` header: if present and starts with
/// `DynamoDB_`, the request is routed to DynamoDB. All other requests go to S3.
#[derive(Debug)]
pub struct GatewayService<S3H: S3Handler, DDBH: DynamoDBHandler> {
    s3: S3HttpService<S3H>,
    dynamodb: DynamoDBHttpService<DDBH>,
}

impl<S3H: S3Handler, DDBH: DynamoDBHandler> GatewayService<S3H, DDBH> {
    /// Create a new gateway wrapping an S3 service and a DynamoDB service.
    pub fn new(s3: S3HttpService<S3H>, dynamodb: DynamoDBHttpService<DDBH>) -> Self {
        Self { s3, dynamodb }
    }
}

impl<S3H: S3Handler, DDBH: DynamoDBHandler> Clone for GatewayService<S3H, DDBH> {
    fn clone(&self) -> Self {
        Self {
            s3: self.s3.clone(),
            dynamodb: self.dynamodb.clone(),
        }
    }
}

impl<S3H: S3Handler, DDBH: DynamoDBHandler> Service<http::Request<Incoming>>
    for GatewayService<S3H, DDBH>
{
    type Response = http::Response<GatewayBody>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: http::Request<Incoming>) -> Self::Future {
        // Intercept health checks at the gateway level.
        if is_health_check(req.method(), req.uri().path()) {
            return Box::pin(async { Ok(health_check_response()) });
        }

        if is_dynamodb_request(&req) {
            let ddb = self.dynamodb.clone();
            return Box::pin(async move {
                let resp = ddb.call(req).await;
                // Both services use Infallible, so the result is always Ok.
                Ok(resp.unwrap_or_else(|e| match e {}).map(Either::Right))
            });
        }

        let s3 = self.s3.clone();
        Box::pin(async move {
            let resp = s3.call(req).await;
            Ok(resp.unwrap_or_else(|e| match e {}).map(Either::Left))
        })
    }
}

/// Check if a request should be routed to DynamoDB based on the X-Amz-Target header.
fn is_dynamodb_request<B>(req: &http::Request<B>) -> bool {
    req.headers()
        .get("x-amz-target")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|t| t.starts_with("DynamoDB_"))
}

/// Check if the request is a health check probe.
fn is_health_check(method: &http::Method, path: &str) -> bool {
    *method == http::Method::GET
        && (path == "/_localstack/health"
            || path == "/_health"
            || path == "/health"
            || path == "/minio/health/live"
            || path == "/minio/health/ready"
            || path == "/minio/health/cluster")
}

/// Produce a combined health check response for all services.
fn health_check_response() -> http::Response<GatewayBody> {
    let body = r#"{"services":{"s3":"running","dynamodb":"running"}}"#;
    http::Response::builder()
        .status(http::StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Either::Left(S3ResponseBody::from_string(body)))
        .expect("static health response should be valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_detect_dynamodb_request() {
        let req = http::Request::builder()
            .header("x-amz-target", "DynamoDB_20120810.CreateTable")
            .body(())
            .unwrap();
        assert!(is_dynamodb_request(&req));
    }

    #[test]
    fn test_should_not_detect_s3_request_as_dynamodb() {
        let req = http::Request::builder().body(()).unwrap();
        assert!(!is_dynamodb_request(&req));
    }

    #[test]
    fn test_should_not_detect_other_target_as_dynamodb() {
        let req = http::Request::builder()
            .header("x-amz-target", "SomeOtherService.Action")
            .body(())
            .unwrap();
        assert!(!is_dynamodb_request(&req));
    }

    #[test]
    fn test_should_detect_health_check_paths() {
        assert!(is_health_check(&http::Method::GET, "/_localstack/health"));
        assert!(is_health_check(&http::Method::GET, "/_health"));
        assert!(is_health_check(&http::Method::GET, "/health"));
        assert!(!is_health_check(&http::Method::POST, "/_health"));
        assert!(!is_health_check(&http::Method::GET, "/mybucket"));
    }

    #[test]
    fn test_should_produce_health_check_response_with_both_services() {
        let resp = health_check_response();
        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("Content-Type")
                .and_then(|v| v.to_str().ok()),
            Some("application/json"),
        );
    }
}
