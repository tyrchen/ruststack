//! Message operation output types.

use serde::{Deserialize, Serialize};

use crate::types::Message;

/// Output for `SendMessage`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendMessageOutput {
    /// The message ID assigned by SQS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// MD5 digest of the message body.
    #[serde(rename = "MD5OfMessageBody")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5_of_message_body: Option<String>,
    /// MD5 digest of the message attributes.
    #[serde(rename = "MD5OfMessageAttributes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5_of_message_attributes: Option<String>,
    /// MD5 digest of the message system attributes.
    #[serde(rename = "MD5OfMessageSystemAttributes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5_of_message_system_attributes: Option<String>,
    /// Sequence number (FIFO queues only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence_number: Option<String>,
}

/// Output for `ReceiveMessage`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReceiveMessageOutput {
    /// List of received messages.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<Message>,
}

/// Output for `DeleteMessage` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeleteMessageOutput {}
