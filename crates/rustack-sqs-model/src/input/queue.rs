//! Queue management input types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Input for `CreateQueue`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateQueueInput {
    /// The queue name.
    pub queue_name: String,
    /// Queue attributes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
    /// Tags for the queue.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tags: HashMap<String, String>,
}

/// Input for `DeleteQueue`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteQueueInput {
    /// The URL of the queue to delete.
    pub queue_url: String,
}

/// Input for `GetQueueUrl`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetQueueUrlInput {
    /// The queue name.
    pub queue_name: String,
    /// The account ID that created the queue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_owner_aws_account_id: Option<String>,
}

/// Input for `ListQueues`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListQueuesInput {
    /// Filter queues by name prefix.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_name_prefix: Option<String>,
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,
}

/// Input for `GetQueueAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetQueueAttributesInput {
    /// The queue URL.
    pub queue_url: String,
    /// List of attribute names to retrieve.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attribute_names: Vec<String>,
}

/// Input for `SetQueueAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetQueueAttributesInput {
    /// The queue URL.
    pub queue_url: String,
    /// Attributes to set.
    pub attributes: HashMap<String, String>,
}

/// Input for `PurgeQueue`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PurgeQueueInput {
    /// The queue URL.
    pub queue_url: String,
}
