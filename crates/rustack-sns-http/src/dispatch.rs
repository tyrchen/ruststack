//! SNS handler trait and operation dispatch.

use std::{future::Future, pin::Pin};

use bytes::Bytes;
use rustack_sns_model::{error::SnsError, operations::SnsOperation};

use crate::body::SnsResponseBody;

/// Trait that the SNS business logic provider must implement.
///
/// The handler receives a parsed operation enum and the raw form body bytes.
/// Unlike SQS/SSM which receive JSON, SNS receives form-urlencoded params.
pub trait SnsHandler: Send + Sync + 'static {
    /// Handle an SNS operation and produce an HTTP response.
    fn handle_operation(
        &self,
        op: SnsOperation,
        body: Bytes,
    ) -> Pin<Box<dyn Future<Output = Result<http::Response<SnsResponseBody>, SnsError>> + Send>>;
}

/// Dispatch an SNS operation to the handler.
pub async fn dispatch_operation<H: SnsHandler>(
    handler: &H,
    op: SnsOperation,
    body: Bytes,
) -> Result<http::Response<SnsResponseBody>, SnsError> {
    tracing::debug!(operation = %op, "dispatching SNS operation");
    handler.handle_operation(op, body).await
}
