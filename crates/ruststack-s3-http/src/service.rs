//! The main S3 HTTP service implementing hyper's `Service` trait.
//!
//! [`S3HttpService`] ties together routing, authentication, dispatch, and response
//! serialization into a single hyper-compatible service. It handles:
//!
//! 1. Health check interception (`GET /_localstack/health`)
//! 2. CORS preflight requests (`OPTIONS`)
//! 3. Request body collection
//! 4. S3 request routing via [`S3Router`]
//! 5. Optional SigV4 authentication
//! 6. Operation dispatch to the [`S3Handler`]
//! 7. Common response headers (`x-amz-request-id`, `Server`, `Date`)
//! 8. Error response formatting

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::service::Service;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::body::S3ResponseBody;
use crate::dispatch::{S3Handler, dispatch_operation};
use crate::response::error_to_response;
use crate::router::S3Router;

/// Configuration for the S3 HTTP service.
#[derive(Debug, Clone)]
pub struct S3HttpConfig {
    /// The base domain for virtual-hosted-style requests (e.g., `s3.localhost`).
    pub domain: String,
    /// Whether to enable virtual-hosted-style bucket addressing.
    pub virtual_hosting: bool,
    /// Whether to skip SigV4 signature validation (useful for development).
    pub skip_signature_validation: bool,
    /// The AWS region this service operates in.
    pub region: String,
}

impl Default for S3HttpConfig {
    fn default() -> Self {
        Self {
            domain: "s3.localhost".to_owned(),
            virtual_hosting: true,
            skip_signature_validation: true,
            region: "us-east-1".to_owned(),
        }
    }
}

/// The S3 HTTP service that implements hyper's `Service` trait.
///
/// This service processes incoming HTTP requests through the full S3 request lifecycle:
/// routing, authentication, dispatch to the handler, and response formatting.
///
/// # Type Parameters
///
/// - `H`: The business logic handler implementing [`S3Handler`].
#[derive(Debug)]
pub struct S3HttpService<H: S3Handler> {
    handler: Arc<H>,
    router: S3Router,
    config: Arc<S3HttpConfig>,
}

impl<H: S3Handler> S3HttpService<H> {
    /// Create a new S3 HTTP service with the given handler and configuration.
    #[must_use]
    pub fn new(handler: H, config: S3HttpConfig) -> Self {
        let router = S3Router::new(&config.domain, config.virtual_hosting);
        Self {
            handler: Arc::new(handler),
            router,
            config: Arc::new(config),
        }
    }

    /// Create a new S3 HTTP service from an `Arc<H>` handler and configuration.
    #[must_use]
    pub fn from_shared(handler: Arc<H>, config: S3HttpConfig) -> Self {
        let router = S3Router::new(&config.domain, config.virtual_hosting);
        Self {
            handler,
            router,
            config: Arc::new(config),
        }
    }
}

impl<H: S3Handler> Clone for S3HttpService<H> {
    fn clone(&self) -> Self {
        Self {
            handler: Arc::clone(&self.handler),
            router: self.router.clone(),
            config: Arc::clone(&self.config),
        }
    }
}

impl<H: S3Handler> Service<http::Request<Incoming>> for S3HttpService<H> {
    type Response = http::Response<S3ResponseBody>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: http::Request<Incoming>) -> Self::Future {
        let handler = Arc::clone(&self.handler);
        let router = self.router.clone();
        let config = Arc::clone(&self.config);

        Box::pin(async move {
            let request_id = Uuid::new_v4().to_string();

            // Process the request through the S3 pipeline.
            let response =
                process_request(req, handler.as_ref(), &router, &config, &request_id).await;

            // Add common response headers.
            let response = add_common_headers(response, &request_id);

            Ok(response)
        })
    }
}

/// Process an incoming HTTP request through the S3 pipeline.
async fn process_request<H: S3Handler>(
    req: http::Request<Incoming>,
    handler: &H,
    router: &S3Router,
    _config: &S3HttpConfig,
    request_id: &str,
) -> http::Response<S3ResponseBody> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    debug!(%method, %uri, request_id, "processing S3 request");

    // 1. Health check interception.
    if is_health_check(&method, uri.path()) {
        return health_check_response();
    }

    // 2. CORS preflight.
    if method == http::Method::OPTIONS {
        return cors_preflight_response();
    }

    // 3. Route the request (needs the original request for headers).
    let routing_result = router.resolve(&req);
    let ctx = match routing_result {
        Ok(ctx) => ctx,
        Err(err) => {
            warn!(
                %method, %uri, error = %err, request_id,
                "failed to route S3 request"
            );
            return error_to_response(&err, request_id);
        }
    };

    info!(
        operation = %ctx.operation,
        bucket = ?ctx.bucket,
        key = ?ctx.key,
        request_id,
        "routed S3 request"
    );

    // 4. Collect body.
    let (parts, incoming) = req.into_parts();
    let body = match collect_body(incoming).await {
        Ok(body) => body,
        Err(err) => {
            error!(error = %err, request_id, "failed to collect request body");
            let s3_err =
                ruststack_s3_model::error::S3Error::internal_error("Failed to read request body");
            return error_to_response(&s3_err, request_id);
        }
    };

    // 5. Authentication (skipped for now - Phase 3).

    // 6. Dispatch to handler.
    match dispatch_operation(handler, parts, body, ctx).await {
        Ok(response) => response,
        Err(err) => {
            debug!(
                error = %err,
                request_id,
                "S3 operation returned error"
            );
            error_to_response(&err, request_id)
        }
    }
}

/// Collect the full body from a hyper `Incoming` stream into `Bytes`.
async fn collect_body(incoming: Incoming) -> Result<Bytes, hyper::Error> {
    let collected = incoming.collect().await?;
    Ok(collected.to_bytes())
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

/// Produce a health check response.
fn health_check_response() -> http::Response<S3ResponseBody> {
    http::Response::builder()
        .status(http::StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(S3ResponseBody::from_string(
            r#"{"status":"running","service":"s3"}"#,
        ))
        .expect("static health response should be valid")
}

/// Produce a CORS preflight response.
fn cors_preflight_response() -> http::Response<S3ResponseBody> {
    http::Response::builder()
        .status(http::StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header(
            "Access-Control-Allow-Methods",
            "GET, PUT, POST, DELETE, HEAD, OPTIONS",
        )
        .header(
            "Access-Control-Allow-Headers",
            "*, Authorization, Content-Type, x-amz-*",
        )
        .header("Access-Control-Max-Age", "86400")
        .body(S3ResponseBody::empty())
        .expect("static CORS response should be valid")
}

/// Add common response headers to every S3 response.
fn add_common_headers(
    mut response: http::Response<S3ResponseBody>,
    request_id: &str,
) -> http::Response<S3ResponseBody> {
    let headers = response.headers_mut();

    // x-amz-request-id
    if let Ok(hv) = http::header::HeaderValue::from_str(request_id) {
        headers.insert("x-amz-request-id", hv);
    }

    // x-amz-id-2 (extended request ID - typically Base64 in real S3)
    if let Ok(hv) = http::header::HeaderValue::from_str(request_id) {
        headers.insert("x-amz-id-2", hv);
    }

    // Server header
    headers.insert(
        "Server",
        http::header::HeaderValue::from_static("RustStackS3"),
    );

    // CORS headers for all responses
    headers.insert(
        "Access-Control-Allow-Origin",
        http::header::HeaderValue::from_static("*"),
    );
    headers.insert(
        "Access-Control-Expose-Headers",
        http::header::HeaderValue::from_static(
            "x-amz-request-id, x-amz-id-2, x-amz-version-id, ETag, x-amz-delete-marker",
        ),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_detect_health_check_paths() {
        assert!(is_health_check(&http::Method::GET, "/_localstack/health"));
        assert!(is_health_check(&http::Method::GET, "/_health"));
        assert!(is_health_check(&http::Method::GET, "/health"));
        assert!(!is_health_check(&http::Method::POST, "/_health"));
        assert!(!is_health_check(&http::Method::GET, "/mybucket"));
    }

    #[test]
    fn test_should_produce_health_check_response() {
        let resp = health_check_response();
        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("Content-Type")
                .and_then(|v| v.to_str().ok()),
            Some("application/json"),
        );
    }

    #[test]
    fn test_should_produce_cors_preflight_response() {
        let resp = cors_preflight_response();
        assert_eq!(resp.status(), http::StatusCode::OK);
        assert!(resp.headers().contains_key("Access-Control-Allow-Origin"));
        assert!(resp.headers().contains_key("Access-Control-Allow-Methods"));
    }

    #[test]
    fn test_should_add_common_headers() {
        let resp = http::Response::builder()
            .status(http::StatusCode::OK)
            .body(S3ResponseBody::empty())
            .expect("valid response");
        let resp = add_common_headers(resp, "test-request-id");
        assert_eq!(
            resp.headers()
                .get("x-amz-request-id")
                .and_then(|v| v.to_str().ok()),
            Some("test-request-id"),
        );
        assert_eq!(
            resp.headers().get("Server").and_then(|v| v.to_str().ok()),
            Some("RustStackS3"),
        );
    }

    #[test]
    fn test_should_create_default_config() {
        let config = S3HttpConfig::default();
        assert_eq!(config.domain, "s3.localhost");
        assert!(config.virtual_hosting);
        assert!(config.skip_signature_validation);
        assert_eq!(config.region, "us-east-1");
    }
}
