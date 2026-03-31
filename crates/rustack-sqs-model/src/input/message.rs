//! Message operation input types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::{MessageAttributeValue, MessageSystemAttributeValue};

/// Input for `SendMessage`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendMessageInput {
    /// The queue URL.
    pub queue_url: String,
    /// The message body.
    pub message_body: String,
    /// Per-message delay in seconds (0-900).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_seconds: Option<i32>,
    /// Message deduplication ID (FIFO only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_deduplication_id: Option<String>,
    /// Message group ID (FIFO only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_group_id: Option<String>,
    /// User-defined message attributes (up to 10).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub message_attributes: HashMap<String, MessageAttributeValue>,
    /// System message attributes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub message_system_attributes: HashMap<String, MessageSystemAttributeValue>,
}

/// Input for `ReceiveMessage`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReceiveMessageInput {
    /// The queue URL.
    pub queue_url: String,
    /// Maximum number of messages to receive (1-10, default 1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_number_of_messages: Option<i32>,
    /// Visibility timeout for received messages (0-43200).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility_timeout: Option<i32>,
    /// Wait time in seconds for long polling (0-20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_time_seconds: Option<i32>,
    /// System attribute names to include in results.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attribute_names: Vec<String>,
    /// System attribute names to include (newer API).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub message_system_attribute_names: Vec<String>,
    /// User-defined attribute names to include in results.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub message_attribute_names: Vec<String>,
    /// Receive request attempt ID (FIFO queues).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receive_request_attempt_id: Option<String>,
}

/// Input for `DeleteMessage`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteMessageInput {
    /// The queue URL.
    pub queue_url: String,
    /// Receipt handle of the message to delete.
    pub receipt_handle: String,
}
