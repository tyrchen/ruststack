//! SES v2 HTTP handler (restJson1 protocol).
//!
//! SES v2 uses path-based routing under `/v2/email/` with JSON request/response bodies.

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use http_body_util::BodyExt;
use hyper::body::Incoming;

use ruststack_ses_model::error::SesError;

use crate::body::SesResponseBody;
use crate::dispatch::SesHandler;
use crate::request::parse_query_params;
use crate::response::{JSON_CONTENT_TYPE, error_to_json_response, json_response};

/// Hyper `Service` implementation for SES v2 (restJson1).
#[derive(Debug)]
pub struct SesV2HttpService<H: SesHandler> {
    handler: Arc<H>,
}

impl<H: SesHandler> SesV2HttpService<H> {
    /// Create a new `SesV2HttpService`.
    pub fn new(handler: Arc<H>) -> Self {
        Self { handler }
    }
}

impl<H: SesHandler> Clone for SesV2HttpService<H> {
    fn clone(&self) -> Self {
        Self {
            handler: Arc::clone(&self.handler),
        }
    }
}

impl<H: SesHandler> hyper::service::Service<http::Request<Incoming>> for SesV2HttpService<H> {
    type Response = http::Response<SesResponseBody>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: http::Request<Incoming>) -> Self::Future {
        let handler = Arc::clone(&self.handler);

        Box::pin(async move {
            let method = req.method().clone();
            let uri = req.uri().clone();
            let path = uri.path().to_owned();
            let query = uri.query().map(str::to_owned);

            let body = match req
                .into_body()
                .collect()
                .await
                .map(http_body_util::Collected::to_bytes)
            {
                Ok(body) => body,
                Err(e) => {
                    let err = SesError::internal_error(format!("Failed to read body: {e}"));
                    return Ok(error_to_json_response(&err));
                }
            };

            // Handle retrospection endpoints.
            if path.starts_with("/_aws/ses") {
                let query_params = parse_query_params(query.as_deref());
                return Ok(handle_retrospection(
                    handler.as_ref(),
                    &method,
                    &query_params,
                ));
            }

            // All other paths are SES v2 operations.
            let response = match handler.handle_v2_operation(method, path, body).await {
                Ok(resp) => resp,
                Err(err) => error_to_json_response(&err),
            };

            Ok(add_v2_headers(response))
        })
    }
}

/// Handle retrospection endpoints (`/_aws/ses`).
fn handle_retrospection<H: SesHandler>(
    handler: &H,
    method: &http::Method,
    query_params: &std::collections::HashMap<String, String>,
) -> http::Response<SesResponseBody> {
    match *method {
        http::Method::GET => {
            let filter_id = query_params.get("id").map(String::as_str);
            let filter_source = query_params.get("email").map(String::as_str);
            let json = handler.query_emails(filter_id, filter_source);
            json_response(json, http::StatusCode::OK)
        }
        http::Method::DELETE => {
            let filter_id = query_params.get("id").map(String::as_str);
            handler.clear_emails(filter_id);
            json_response("{}".to_owned(), http::StatusCode::OK)
        }
        _ => {
            let err = SesError::invalid_parameter_value(format!(
                "Method {method} not supported on /_aws/ses"
            ));
            error_to_json_response(&err)
        }
    }
}

/// Add common headers to SES v2 responses.
fn add_v2_headers(
    mut response: http::Response<SesResponseBody>,
) -> http::Response<SesResponseBody> {
    let headers = response.headers_mut();

    headers
        .entry("content-type")
        .or_insert(http::HeaderValue::from_static(JSON_CONTENT_TYPE));

    headers.insert("server", http::HeaderValue::from_static("RustStack"));

    headers.insert(
        "access-control-allow-origin",
        http::HeaderValue::from_static("*"),
    );

    response
}
