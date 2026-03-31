//! Shared SNS types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A resource tag (key-value pair).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Tag {
    /// The tag key.
    pub key: String,
    /// The tag value.
    pub value: String,
}

/// A message attribute value.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MessageAttributeValue {
    /// The data type: `String`, `Number`, `Binary`, or custom types.
    pub data_type: String,
    /// The string value (for `String` and `Number` types).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_value: Option<String>,
    /// The binary value (base64 encoded).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_value: Option<String>,
}

/// A subscription summary for list results.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Subscription {
    /// The subscription ARN.
    pub subscription_arn: String,
    /// The subscription owner account ID.
    pub owner: String,
    /// The subscription protocol (e.g., `http`, `https`, `email`, `sqs`, `lambda`).
    pub protocol: String,
    /// The subscription endpoint.
    pub endpoint: String,
    /// The topic ARN.
    pub topic_arn: String,
}

/// A topic summary for list results.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Topic {
    /// The topic ARN.
    pub topic_arn: String,
}

/// A successful result for one entry in a `PublishBatch` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PublishBatchResultEntry {
    /// The caller-chosen ID for this entry.
    pub id: String,
    /// The message ID assigned by SNS.
    pub message_id: String,
    /// The sequence number (FIFO topics only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence_number: Option<String>,
}

/// A failed result for one entry in a batch operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BatchResultErrorEntry {
    /// The caller-chosen ID for this entry.
    pub id: String,
    /// The error code.
    pub code: String,
    /// The error message.
    pub message: String,
    /// Whether the error is a sender fault.
    pub sender_fault: bool,
}

/// A single entry in a `PublishBatch` request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PublishBatchRequestEntry {
    /// The caller-chosen ID for this entry (unique within the batch).
    pub id: String,
    /// The message body.
    pub message: String,
    /// The message subject.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The message structure (e.g., `json` for per-protocol messages).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_structure: Option<String>,
    /// User-defined message attributes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub message_attributes: HashMap<String, MessageAttributeValue>,
    /// Message deduplication ID (FIFO topics only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_deduplication_id: Option<String>,
    /// Message group ID (FIFO topics only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_group_id: Option<String>,
}

/// A platform application summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlatformApplication {
    /// The platform application ARN.
    pub platform_application_arn: String,
    /// The platform application attributes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

/// A platform endpoint summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Endpoint {
    /// The endpoint ARN.
    pub endpoint_arn: String,
    /// The endpoint attributes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

/// Phone number information for origination numbers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PhoneNumberInformation {
    /// The phone number.
    pub phone_number: String,
    /// The phone number status.
    pub status: String,
    /// The route type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_type: Option<String>,
}

/// An SMS sandbox phone number.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SMSSandboxPhoneNumber {
    /// The phone number.
    pub phone_number: String,
    /// The verification status.
    pub status: String,
}
