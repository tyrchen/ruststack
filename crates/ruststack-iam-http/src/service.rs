//! IAM HTTP service implementing the hyper `Service` trait.
//!
//! IAM uses the `awsQuery` protocol where the request body is
//! `application/x-www-form-urlencoded` and the response is `text/xml`.
//! Like SNS, IAM uses the `Action=` form parameter for operation routing.

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::body::Incoming;

use ruststack_iam_model::error::IamError;

use crate::body::IamResponseBody;
use crate::dispatch::{IamHandler, dispatch_operation};
use crate::request::parse_form_params;
use crate::response::{CONTENT_TYPE, error_to_response};
use crate::router::resolve_operation;

/// Configuration for the IAM HTTP service.
#[derive(Clone)]
pub struct IamHttpConfig {
    /// Whether to skip AWS signature validation.
    pub skip_signature_validation: bool,
    /// The AWS region this service is running in.
    pub region: String,
    /// Credential provider for signature validation.
    pub credential_provider: Option<Arc<dyn ruststack_auth::CredentialProvider>>,
}

impl std::fmt::Debug for IamHttpConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IamHttpConfig")
            .field("skip_signature_validation", &self.skip_signature_validation)
            .field("region", &self.region)
            .field(
                "credential_provider",
                &self.credential_provider.as_ref().map(|_| "..."),
            )
            .finish()
    }
}

impl Default for IamHttpConfig {
    fn default() -> Self {
        Self {
            skip_signature_validation: true,
            region: "us-east-1".to_owned(),
            credential_provider: None,
        }
    }
}

/// Hyper `Service` implementation for IAM.
///
/// Wraps an [`IamHandler`] implementation and routes incoming HTTP
/// requests to the appropriate IAM operation handler.
#[derive(Debug)]
pub struct IamHttpService<H: IamHandler> {
    handler: Arc<H>,
    config: Arc<IamHttpConfig>,
}

impl<H: IamHandler> IamHttpService<H> {
    /// Create a new `IamHttpService`.
    pub fn new(handler: Arc<H>, config: IamHttpConfig) -> Self {
        Self {
            handler,
            config: Arc::new(config),
        }
    }
}

impl<H: IamHandler> Clone for IamHttpService<H> {
    fn clone(&self) -> Self {
        Self {
            handler: Arc::clone(&self.handler),
            config: Arc::clone(&self.config),
        }
    }
}

impl<H: IamHandler> hyper::service::Service<http::Request<Incoming>> for IamHttpService<H> {
    type Response = http::Response<IamResponseBody>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: http::Request<Incoming>) -> Self::Future {
        let handler = Arc::clone(&self.handler);
        let config = Arc::clone(&self.config);
        let request_id = uuid::Uuid::new_v4().to_string();

        Box::pin(async move {
            let response = process_request(req, handler.as_ref(), &config, &request_id).await;
            let response = add_common_headers(response, &request_id);
            Ok(response)
        })
    }
}

/// Process a single IAM HTTP request through the full pipeline.
///
/// Pipeline:
/// 1. Verify POST method (IAM only accepts POST)
/// 2. Collect body
/// 3. Parse form params from body
/// 4. Resolve operation from `Action=` param
/// 5. Authenticate (if enabled)
/// 6. Dispatch to handler (pass raw body bytes)
async fn process_request<H: IamHandler>(
    req: http::Request<Incoming>,
    handler: &H,
    config: &IamHttpConfig,
    request_id: &str,
) -> http::Response<IamResponseBody> {
    let (parts, incoming) = req.into_parts();

    // 1. Verify POST method (IAM only accepts POST).
    if parts.method != http::Method::POST {
        let err =
            IamError::invalid_input(format!("IAM requires POST method, got {}", parts.method));
        return error_to_response(&err, request_id);
    }

    // 2. Collect body.
    let body = match collect_body(incoming).await {
        Ok(body) => body,
        Err(err) => return error_to_response(&err, request_id),
    };

    // 3. Parse form params to extract Action for routing.
    let params = parse_form_params(&body);

    // 4. Resolve operation from Action= param.
    let op = match resolve_operation(&params) {
        Ok(op) => op,
        Err(err) => return error_to_response(&err, request_id),
    };

    // 5. Authenticate (if enabled).
    if !config.skip_signature_validation {
        if let Some(ref cred_provider) = config.credential_provider {
            let body_hash = ruststack_auth::hash_payload(&body);
            if let Err(auth_err) =
                ruststack_auth::verify_sigv4(&parts, &body_hash, cred_provider.as_ref())
            {
                let err = IamError::invalid_security(auth_err.to_string());
                return error_to_response(&err, request_id);
            }
        }
    }

    // 6. Dispatch to handler (pass raw body so handler can re-parse as needed).
    match dispatch_operation(handler, op, body).await {
        Ok(response) => response,
        Err(err) => error_to_response(&err, request_id),
    }
}

/// Collect the incoming body into a single `Bytes` buffer.
async fn collect_body(incoming: Incoming) -> Result<Bytes, IamError> {
    incoming
        .collect()
        .await
        .map(http_body_util::Collected::to_bytes)
        .map_err(|e| IamError::internal_error(format!("Failed to read request body: {e}")))
}

/// Add common response headers to every IAM response.
fn add_common_headers(
    mut response: http::Response<IamResponseBody>,
    request_id: &str,
) -> http::Response<IamResponseBody> {
    let headers = response.headers_mut();

    if let Ok(hv) = http::HeaderValue::from_str(request_id) {
        headers.entry("x-amzn-requestid").or_insert(hv);
    }

    headers
        .entry("content-type")
        .or_insert(http::HeaderValue::from_static(CONTENT_TYPE));

    headers.insert("server", http::HeaderValue::from_static("RustStack"));

    // CORS headers.
    headers.insert(
        "access-control-allow-origin",
        http::HeaderValue::from_static("*"),
    );

    response
}
