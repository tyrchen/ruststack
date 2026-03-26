//! SQS handler implementation bridging HTTP to business logic.

use std::{future::Future, pin::Pin, sync::Arc};

use bytes::Bytes;
use ruststack_sqs_http::{body::SqsResponseBody, dispatch::SqsHandler, response::json_response};
use ruststack_sqs_model::{error::SqsError, operations::SqsOperation};

use crate::provider::RustStackSqs;

/// Handler that bridges the HTTP layer to the SQS provider.
#[derive(Debug)]
pub struct RustStackSqsHandler {
    provider: Arc<RustStackSqs>,
}

impl RustStackSqsHandler {
    /// Create a new handler wrapping a provider.
    #[must_use]
    pub fn new(provider: Arc<RustStackSqs>) -> Self {
        Self { provider }
    }
}

impl SqsHandler for RustStackSqsHandler {
    fn handle_operation(
        &self,
        op: SqsOperation,
        body: Bytes,
    ) -> Pin<Box<dyn Future<Output = Result<http::Response<SqsResponseBody>, SqsError>> + Send>>
    {
        let provider = Arc::clone(&self.provider);
        Box::pin(async move { dispatch(provider.as_ref(), op, &body).await })
    }
}

/// Dispatch an SQS operation to the appropriate handler method.
#[allow(clippy::too_many_lines)] // Match dispatch with one arm per SQS operation.
async fn dispatch(
    provider: &RustStackSqs,
    op: SqsOperation,
    body: &[u8],
) -> Result<http::Response<SqsResponseBody>, SqsError> {
    let request_id = uuid::Uuid::new_v4().to_string();

    match op {
        SqsOperation::CreateQueue => {
            let input = deserialize(body)?;
            let output = provider.create_queue(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::DeleteQueue => {
            let input = deserialize(body)?;
            let output = provider.delete_queue(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::GetQueueUrl => {
            let input = deserialize(body)?;
            let output = provider.get_queue_url(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::ListQueues => {
            let input = deserialize(body)?;
            let output = provider.list_queues(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::GetQueueAttributes => {
            let input = deserialize(body)?;
            let output = provider.get_queue_attributes(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::SetQueueAttributes => {
            let input = deserialize(body)?;
            let output = provider.set_queue_attributes(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::SendMessage => {
            let input = deserialize(body)?;
            let output = provider.send_message(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::ReceiveMessage => {
            let input = deserialize(body)?;
            let output = provider.receive_message(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::DeleteMessage => {
            let input = deserialize(body)?;
            let output = provider.delete_message(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::PurgeQueue => {
            let input = deserialize(body)?;
            let output = provider.purge_queue(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::SendMessageBatch => {
            let input = deserialize(body)?;
            let output = provider.send_message_batch(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::DeleteMessageBatch => {
            let input = deserialize(body)?;
            let output = provider.delete_message_batch(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::ChangeMessageVisibility => {
            let input = deserialize(body)?;
            let output = provider.change_message_visibility(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::ChangeMessageVisibilityBatch => {
            let input = deserialize(body)?;
            let output = provider.change_message_visibility_batch(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::TagQueue => {
            let input = deserialize(body)?;
            let output = provider.tag_queue(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::UntagQueue => {
            let input = deserialize(body)?;
            let output = provider.untag_queue(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::ListQueueTags => {
            let input = deserialize(body)?;
            let output = provider.list_queue_tags(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::AddPermission => {
            let input = deserialize(body)?;
            let output = provider.add_permission(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::RemovePermission => {
            let input = deserialize(body)?;
            let output = provider.remove_permission(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::ListDeadLetterSourceQueues => {
            let input = deserialize(body)?;
            let output = provider.list_dead_letter_source_queues(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::StartMessageMoveTask => {
            let input = deserialize(body)?;
            let output = provider.start_message_move_task(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::CancelMessageMoveTask => {
            let input = deserialize(body)?;
            let output = provider.cancel_message_move_task(input).await?;
            serialize(&output, &request_id)
        }
        SqsOperation::ListMessageMoveTasks => {
            let input = deserialize(body)?;
            let output = provider.list_message_move_tasks(input).await?;
            serialize(&output, &request_id)
        }
    }
}

/// Deserialize a JSON request body into the input type.
fn deserialize<T: serde::de::DeserializeOwned>(body: &[u8]) -> Result<T, SqsError> {
    serde_json::from_slice(body).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("missing field") {
            SqsError::missing_parameter(format!("1 validation error detected: {msg}"))
        } else {
            SqsError::invalid_parameter_value(format!("Failed to deserialize request body: {e}"))
        }
    })
}

/// Serialize an output type into a JSON HTTP response.
fn serialize<T: serde::Serialize>(
    output: &T,
    request_id: &str,
) -> Result<http::Response<SqsResponseBody>, SqsError> {
    let json = serde_json::to_vec(output)
        .map_err(|e| SqsError::internal_error(format!("Failed to serialize response: {e}")))?;
    Ok(json_response(json, request_id))
}
