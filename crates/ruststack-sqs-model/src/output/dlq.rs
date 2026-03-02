//! Dead-letter queue operation output types.

use serde::{Deserialize, Serialize};

use crate::types::ListMessageMoveTasksResultEntry;

/// Output for `ListDeadLetterSourceQueues`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListDeadLetterSourceQueuesOutput {
    /// Queue URLs that have a redrive policy targeting the given DLQ.
    pub queue_urls: Vec<String>,
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// Output for `StartMessageMoveTask`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StartMessageMoveTaskOutput {
    /// The task handle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_handle: Option<String>,
}

/// Output for `CancelMessageMoveTask`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CancelMessageMoveTaskOutput {
    /// Approximate number of messages already moved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approximate_number_of_messages_moved: Option<i64>,
}

/// Output for `ListMessageMoveTasks`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListMessageMoveTasksOutput {
    /// Task results.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<ListMessageMoveTasksResultEntry>,
}
