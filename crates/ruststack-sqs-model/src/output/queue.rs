//! Queue management output types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Output for `CreateQueue`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateQueueOutput {
    /// The URL of the created queue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_url: Option<String>,
}

/// Output for `DeleteQueue` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeleteQueueOutput {}

/// Output for `GetQueueUrl`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetQueueUrlOutput {
    /// The queue URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_url: Option<String>,
}

/// Output for `ListQueues`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListQueuesOutput {
    /// Queue URLs matching the filter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub queue_urls: Vec<String>,
    /// Pagination token for next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// Output for `GetQueueAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetQueueAttributesOutput {
    /// Requested attributes as key-value pairs.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

/// Output for `SetQueueAttributes` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SetQueueAttributesOutput {}

/// Output for `PurgeQueue` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PurgeQueueOutput {}
