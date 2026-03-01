//! DynamoDB handler implementation bridging HTTP to business logic.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;

use ruststack_dynamodb_http::body::DynamoDBResponseBody;
use ruststack_dynamodb_http::dispatch::DynamoDBHandler;
use ruststack_dynamodb_http::response::json_response;
use ruststack_dynamodb_model::error::DynamoDBError;
use ruststack_dynamodb_model::operations::DynamoDBOperation;

use crate::provider::RustStackDynamoDB;

/// Handler that bridges the HTTP layer to the DynamoDB provider.
#[derive(Debug)]
pub struct RustStackDynamoDBHandler {
    provider: Arc<RustStackDynamoDB>,
}

impl RustStackDynamoDBHandler {
    /// Create a new handler wrapping a provider.
    #[must_use]
    pub fn new(provider: Arc<RustStackDynamoDB>) -> Self {
        Self { provider }
    }
}

impl DynamoDBHandler for RustStackDynamoDBHandler {
    fn handle_operation(
        &self,
        op: DynamoDBOperation,
        body: Bytes,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<http::Response<DynamoDBResponseBody>, DynamoDBError>> + Send,
        >,
    > {
        let provider = Arc::clone(&self.provider);
        Box::pin(async move { dispatch(provider.as_ref(), op, &body) })
    }
}

/// Dispatch a DynamoDB operation to the appropriate handler method.
fn dispatch(
    provider: &RustStackDynamoDB,
    op: DynamoDBOperation,
    body: &[u8],
) -> Result<http::Response<DynamoDBResponseBody>, DynamoDBError> {
    // Generate a request ID for responses.
    let request_id = uuid::Uuid::new_v4().to_string();

    match op {
        DynamoDBOperation::CreateTable => {
            let input = deserialize(body)?;
            let output = provider.handle_create_table(input)?;
            serialize(&output, &request_id)
        }
        DynamoDBOperation::DeleteTable => {
            let input = deserialize(body)?;
            let output = provider.handle_delete_table(input)?;
            serialize(&output, &request_id)
        }
        DynamoDBOperation::DescribeTable => {
            let input = deserialize(body)?;
            let output = provider.handle_describe_table(input)?;
            serialize(&output, &request_id)
        }
        DynamoDBOperation::ListTables => {
            let input = deserialize(body)?;
            let output = provider.handle_list_tables(input)?;
            serialize(&output, &request_id)
        }
        DynamoDBOperation::PutItem => {
            let input = deserialize(body)?;
            let output = provider.handle_put_item(input)?;
            serialize(&output, &request_id)
        }
        DynamoDBOperation::GetItem => {
            let input = deserialize(body)?;
            let output = provider.handle_get_item(input)?;
            serialize(&output, &request_id)
        }
        DynamoDBOperation::UpdateItem => {
            let input = deserialize(body)?;
            let output = provider.handle_update_item(input)?;
            serialize(&output, &request_id)
        }
        DynamoDBOperation::DeleteItem => {
            let input = deserialize(body)?;
            let output = provider.handle_delete_item(input)?;
            serialize(&output, &request_id)
        }
        DynamoDBOperation::Query => {
            let input = deserialize(body)?;
            let output = provider.handle_query(input)?;
            serialize(&output, &request_id)
        }
        DynamoDBOperation::Scan => {
            let input = deserialize(body)?;
            let output = provider.handle_scan(input)?;
            serialize(&output, &request_id)
        }
        DynamoDBOperation::BatchGetItem => {
            let input = deserialize(body)?;
            let output = provider.handle_batch_get_item(input)?;
            serialize(&output, &request_id)
        }
        DynamoDBOperation::BatchWriteItem => {
            let input = deserialize(body)?;
            let output = provider.handle_batch_write_item(input)?;
            serialize(&output, &request_id)
        }
    }
}

/// Deserialize a JSON request body into the input type.
fn deserialize<T: serde::de::DeserializeOwned>(body: &[u8]) -> Result<T, DynamoDBError> {
    serde_json::from_slice(body).map_err(|e| {
        DynamoDBError::serialization_exception(format!("Failed to deserialize request body: {e}"))
    })
}

/// Serialize an output type into a JSON HTTP response.
fn serialize<T: serde::Serialize>(
    output: &T,
    request_id: &str,
) -> Result<http::Response<DynamoDBResponseBody>, DynamoDBError> {
    let json = serde_json::to_vec(output)
        .map_err(|e| DynamoDBError::internal_error(format!("Failed to serialize response: {e}")))?;
    Ok(json_response(json, request_id))
}
