//! Tag operation output types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Output for `TagQueue` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TagQueueOutput {}

/// Output for `UntagQueue` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UntagQueueOutput {}

/// Output for `ListQueueTags`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListQueueTagsOutput {
    /// Queue tags.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tags: HashMap<String, String>,
}
