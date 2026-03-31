//! Batch operation input types.

use serde::{Deserialize, Serialize};

use crate::types::{
    ChangeMessageVisibilityBatchRequestEntry, DeleteMessageBatchRequestEntry,
    SendMessageBatchRequestEntry,
};

/// Input for `SendMessageBatch`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendMessageBatchInput {
    /// The queue URL.
    pub queue_url: String,
    /// Batch entries (1-10).
    pub entries: Vec<SendMessageBatchRequestEntry>,
}

/// Input for `DeleteMessageBatch`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteMessageBatchInput {
    /// The queue URL.
    pub queue_url: String,
    /// Batch entries (1-10).
    pub entries: Vec<DeleteMessageBatchRequestEntry>,
}

/// Input for `ChangeMessageVisibilityBatch`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChangeMessageVisibilityBatchInput {
    /// The queue URL.
    pub queue_url: String,
    /// Batch entries (1-10).
    pub entries: Vec<ChangeMessageVisibilityBatchRequestEntry>,
}
