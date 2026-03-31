//! SQS handler trait and operation dispatch.

use std::{future::Future, pin::Pin};

use bytes::Bytes;
use rustack_sqs_model::{error::SqsError, operations::SqsOperation};

use crate::body::SqsResponseBody;

/// Trait that the SQS business logic provider must implement.
///
/// The handler receives a parsed operation enum and the raw JSON body bytes,
/// and returns a complete HTTP response. This trait serves as the boundary
/// between the HTTP transport layer and the business logic layer.
pub trait SqsHandler: Send + Sync + 'static {
    /// Handle an SQS operation and produce an HTTP response.
    fn handle_operation(
        &self,
        op: SqsOperation,
        body: Bytes,
    ) -> Pin<Box<dyn Future<Output = Result<http::Response<SqsResponseBody>, SqsError>> + Send>>;
}

/// Dispatch an SQS operation to the handler.
pub async fn dispatch_operation<H: SqsHandler>(
    handler: &H,
    op: SqsOperation,
    body: Bytes,
) -> Result<http::Response<SqsResponseBody>, SqsError> {
    tracing::debug!(operation = %op, "dispatching SQS operation");
    handler.handle_operation(op, body).await
}
