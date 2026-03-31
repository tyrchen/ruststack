//! Batch operation output types.

use serde::{Deserialize, Serialize};

use crate::types::{
    BatchResultErrorEntry, ChangeMessageVisibilityBatchResultEntry, DeleteMessageBatchResultEntry,
    SendMessageBatchResultEntry,
};

/// Output for `SendMessageBatch`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendMessageBatchOutput {
    /// Successful entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub successful: Vec<SendMessageBatchResultEntry>,
    /// Failed entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failed: Vec<BatchResultErrorEntry>,
}

/// Output for `DeleteMessageBatch`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteMessageBatchOutput {
    /// Successful entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub successful: Vec<DeleteMessageBatchResultEntry>,
    /// Failed entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failed: Vec<BatchResultErrorEntry>,
}

/// Output for `ChangeMessageVisibilityBatch`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChangeMessageVisibilityBatchOutput {
    /// Successful entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub successful: Vec<ChangeMessageVisibilityBatchResultEntry>,
    /// Failed entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failed: Vec<BatchResultErrorEntry>,
}
