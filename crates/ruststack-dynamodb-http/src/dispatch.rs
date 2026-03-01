//! DynamoDB handler trait and operation dispatch.

use std::future::Future;
use std::pin::Pin;

use bytes::Bytes;

use ruststack_dynamodb_model::error::DynamoDBError;
use ruststack_dynamodb_model::operations::DynamoDBOperation;

use crate::body::DynamoDBResponseBody;

/// Trait that the DynamoDB business logic provider must implement.
///
/// The handler receives a parsed operation enum and the raw JSON body bytes,
/// and returns a complete HTTP response. This trait serves as the boundary
/// between the HTTP transport layer and the business logic layer.
pub trait DynamoDBHandler: Send + Sync + 'static {
    /// Handle a DynamoDB operation and produce an HTTP response.
    fn handle_operation(
        &self,
        op: DynamoDBOperation,
        body: Bytes,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<http::Response<DynamoDBResponseBody>, DynamoDBError>> + Send,
        >,
    >;
}

/// Dispatch a DynamoDB operation to the handler.
pub async fn dispatch_operation<H: DynamoDBHandler>(
    handler: &H,
    op: DynamoDBOperation,
    body: Bytes,
) -> Result<http::Response<DynamoDBResponseBody>, DynamoDBError> {
    tracing::debug!(operation = %op, "dispatching DynamoDB operation");
    handler.handle_operation(op, body).await
}

/// Default handler that returns an error for all operations.
#[derive(Debug, Clone, Default)]
pub struct NotImplementedHandler;

impl DynamoDBHandler for NotImplementedHandler {
    fn handle_operation(
        &self,
        op: DynamoDBOperation,
        _body: Bytes,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<http::Response<DynamoDBResponseBody>, DynamoDBError>> + Send,
        >,
    > {
        Box::pin(async move { Err(DynamoDBError::unknown_operation(op.as_str())) })
    }
}
