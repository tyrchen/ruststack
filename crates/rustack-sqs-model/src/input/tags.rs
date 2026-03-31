//! Tag operation input types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Input for `TagQueue`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TagQueueInput {
    /// The queue URL.
    pub queue_url: String,
    /// Tags to add or update.
    pub tags: HashMap<String, String>,
}

/// Input for `UntagQueue`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UntagQueueInput {
    /// The queue URL.
    pub queue_url: String,
    /// Tag keys to remove.
    pub tag_keys: Vec<String>,
}

/// Input for `ListQueueTags`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListQueueTagsInput {
    /// The queue URL.
    pub queue_url: String,
}
