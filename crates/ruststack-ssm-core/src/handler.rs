//! SSM handler implementation bridging HTTP to business logic.

use std::{future::Future, pin::Pin, sync::Arc};

use bytes::Bytes;
use ruststack_ssm_http::{body::SsmResponseBody, dispatch::SsmHandler, response::json_response};
use ruststack_ssm_model::{error::SsmError, operations::SsmOperation};

use crate::provider::RustStackSsm;

/// Handler that bridges the HTTP layer to the SSM provider.
#[derive(Debug)]
pub struct RustStackSsmHandler {
    provider: Arc<RustStackSsm>,
}

impl RustStackSsmHandler {
    /// Create a new handler wrapping a provider.
    #[must_use]
    pub fn new(provider: Arc<RustStackSsm>) -> Self {
        Self { provider }
    }
}

impl SsmHandler for RustStackSsmHandler {
    fn handle_operation(
        &self,
        op: SsmOperation,
        body: Bytes,
    ) -> Pin<Box<dyn Future<Output = Result<http::Response<SsmResponseBody>, SsmError>> + Send>>
    {
        let provider = Arc::clone(&self.provider);
        Box::pin(async move { dispatch(provider.as_ref(), op, &body) })
    }
}

/// Dispatch an SSM operation to the appropriate handler method.
fn dispatch(
    provider: &RustStackSsm,
    op: SsmOperation,
    body: &[u8],
) -> Result<http::Response<SsmResponseBody>, SsmError> {
    // Generate a request ID for responses.
    let request_id = uuid::Uuid::new_v4().to_string();

    if !op.is_implemented() {
        return Err(SsmError::not_implemented(op.as_str()));
    }

    match op {
        // Phase 0
        SsmOperation::PutParameter => {
            let input = deserialize(body)?;
            let output = provider.handle_put_parameter(input)?;
            serialize(&output, &request_id)
        }
        SsmOperation::GetParameter => {
            let input = deserialize(body)?;
            let output = provider.handle_get_parameter(&input)?;
            serialize(&output, &request_id)
        }
        SsmOperation::GetParameters => {
            let input = deserialize(body)?;
            let output = provider.handle_get_parameters(&input)?;
            serialize(&output, &request_id)
        }
        SsmOperation::GetParametersByPath => {
            let input = deserialize(body)?;
            let output = provider.handle_get_parameters_by_path(&input)?;
            serialize(&output, &request_id)
        }
        SsmOperation::DeleteParameter => {
            let input = deserialize(body)?;
            let output = provider.handle_delete_parameter(&input)?;
            serialize(&output, &request_id)
        }
        SsmOperation::DeleteParameters => {
            let input = deserialize(body)?;
            let output = provider.handle_delete_parameters(&input)?;
            serialize(&output, &request_id)
        }
        // Phase 1
        SsmOperation::DescribeParameters => {
            let input = deserialize(body)?;
            let output = provider.handle_describe_parameters(&input)?;
            serialize(&output, &request_id)
        }
        SsmOperation::GetParameterHistory => {
            let input = deserialize(body)?;
            let output = provider.handle_get_parameter_history(&input)?;
            serialize(&output, &request_id)
        }
        SsmOperation::AddTagsToResource => {
            let input = deserialize(body)?;
            let output = provider.handle_add_tags_to_resource(&input)?;
            serialize(&output, &request_id)
        }
        SsmOperation::RemoveTagsFromResource => {
            let input = deserialize(body)?;
            let output = provider.handle_remove_tags_from_resource(&input)?;
            serialize(&output, &request_id)
        }
        SsmOperation::ListTagsForResource => {
            let input = deserialize(body)?;
            let output = provider.handle_list_tags_for_resource(&input)?;
            serialize(&output, &request_id)
        }
        // Phase 2
        SsmOperation::LabelParameterVersion => {
            let input = deserialize(body)?;
            let output = provider.handle_label_parameter_version(&input)?;
            serialize(&output, &request_id)
        }
        SsmOperation::UnlabelParameterVersion => {
            let input = deserialize(body)?;
            let output = provider.handle_unlabel_parameter_version(&input)?;
            serialize(&output, &request_id)
        }
    }
}

/// Deserialize a JSON request body into the input type.
fn deserialize<T: serde::de::DeserializeOwned>(body: &[u8]) -> Result<T, SsmError> {
    serde_json::from_slice(body).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("missing field") || msg.contains("unknown variant") {
            SsmError::validation(format!("1 validation error detected: {msg}"))
        } else {
            SsmError::validation(format!("Failed to deserialize request body: {e}"))
        }
    })
}

/// Serialize an output type into a JSON HTTP response.
fn serialize<T: serde::Serialize>(
    output: &T,
    request_id: &str,
) -> Result<http::Response<SsmResponseBody>, SsmError> {
    let json = serde_json::to_vec(output)
        .map_err(|e| SsmError::internal_error(format!("Failed to serialize response: {e}")))?;
    Ok(json_response(json, request_id))
}
