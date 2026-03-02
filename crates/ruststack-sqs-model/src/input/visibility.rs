//! Visibility timeout input types.

use serde::{Deserialize, Serialize};

/// Input for `ChangeMessageVisibility`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChangeMessageVisibilityInput {
    /// The queue URL.
    pub queue_url: String,
    /// Receipt handle of the message.
    pub receipt_handle: String,
    /// New visibility timeout in seconds (0-43200).
    pub visibility_timeout: i32,
}
