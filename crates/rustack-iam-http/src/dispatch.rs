//! IAM handler trait and operation dispatch.

use std::{future::Future, pin::Pin};

use bytes::Bytes;
use rustack_iam_model::{error::IamError, operations::IamOperation};

use crate::body::IamResponseBody;

/// Trait that the IAM business logic provider must implement.
///
/// The handler receives a parsed operation enum and the raw form body bytes.
/// IAM uses `awsQuery` protocol so it receives form-urlencoded params.
pub trait IamHandler: Send + Sync + 'static {
    /// Handle an IAM operation and produce an HTTP response.
    fn handle_operation(
        &self,
        op: IamOperation,
        body: Bytes,
    ) -> Pin<Box<dyn Future<Output = Result<http::Response<IamResponseBody>, IamError>> + Send>>;
}

/// Dispatch an IAM operation to the handler.
pub async fn dispatch_operation<H: IamHandler>(
    handler: &H,
    op: IamOperation,
    body: Bytes,
) -> Result<http::Response<IamResponseBody>, IamError> {
    tracing::debug!(operation = %op, "dispatching IAM operation");
    handler.handle_operation(op, body).await
}
