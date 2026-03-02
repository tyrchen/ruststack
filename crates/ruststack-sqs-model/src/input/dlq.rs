//! Dead-letter queue operation input types.

use serde::{Deserialize, Serialize};

/// Input for `ListDeadLetterSourceQueues`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListDeadLetterSourceQueuesInput {
    /// The DLQ URL to find source queues for.
    pub queue_url: String,
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,
}

/// Input for `StartMessageMoveTask`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StartMessageMoveTaskInput {
    /// The source ARN (DLQ).
    pub source_arn: String,
    /// The destination ARN (original source queue). If omitted, uses the
    /// source queue from the DLQ's redrive policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_arn: Option<String>,
    /// Maximum messages per second to move.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_number_of_messages_per_second: Option<i32>,
}

/// Input for `CancelMessageMoveTask`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CancelMessageMoveTaskInput {
    /// The task handle to cancel.
    pub task_handle: String,
}

/// Input for `ListMessageMoveTasks`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListMessageMoveTasksInput {
    /// The source ARN to list tasks for.
    pub source_arn: String,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,
}
