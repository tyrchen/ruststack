//! SES handler trait and operation dispatch.

use std::{future::Future, pin::Pin, sync::Arc};

use bytes::Bytes;
use rustack_ses_model::{error::SesError, operations::SesOperation};

use crate::body::SesResponseBody;

/// Trait that the SES business logic provider must implement.
///
/// The handler receives a parsed operation enum and the raw form body bytes.
/// SES v1 receives form-urlencoded params (awsQuery protocol).
pub trait SesHandler: Send + Sync + 'static {
    /// Handle an SES v1 operation and produce an HTTP response.
    fn handle_operation(
        &self,
        op: SesOperation,
        body: Bytes,
    ) -> Pin<Box<dyn Future<Output = Result<http::Response<SesResponseBody>, SesError>> + Send>>;

    /// Handle an SES v2 operation and produce an HTTP response.
    fn handle_v2_operation(
        &self,
        method: http::Method,
        path: String,
        body: Bytes,
    ) -> Pin<Box<dyn Future<Output = Result<http::Response<SesResponseBody>, SesError>> + Send>>;

    /// Query sent emails for the retrospection endpoint.
    ///
    /// Returns a JSON string of `{ "messages": [...] }`.
    fn query_emails(&self, filter_id: Option<&str>, filter_source: Option<&str>) -> String;

    /// Clear sent emails for the retrospection endpoint.
    fn clear_emails(&self, filter_id: Option<&str>);
}

/// Dispatch an SES operation to the handler.
pub async fn dispatch_operation<H: SesHandler>(
    handler: &H,
    op: SesOperation,
    body: Bytes,
) -> Result<http::Response<SesResponseBody>, SesError> {
    tracing::debug!(operation = %op, "dispatching SES operation");
    handler.handle_operation(op, body).await
}

/// Wrapper to hold handler behind Arc for cloning in service layer.
#[derive(Debug, Clone)]
pub struct SesHandlerRef<H: SesHandler> {
    inner: Arc<H>,
}

impl<H: SesHandler> SesHandlerRef<H> {
    /// Create a new handler reference.
    pub fn new(handler: Arc<H>) -> Self {
        Self { inner: handler }
    }

    /// Get the inner handler.
    #[must_use]
    pub fn handler(&self) -> &H {
        &self.inner
    }
}
