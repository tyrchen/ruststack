//! SNS operation input types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::{MessageAttributeValue, PublishBatchRequestEntry, Tag};

// ---------------------------------------------------------------------------
// Topic management
// ---------------------------------------------------------------------------

/// Input for `CreateTopic`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateTopicInput {
    /// The topic name.
    pub name: String,
    /// Topic attributes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
    /// Tags for the topic.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    /// Data protection policy JSON.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_protection_policy: Option<String>,
}

/// Input for `DeleteTopic`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteTopicInput {
    /// The topic ARN.
    pub topic_arn: String,
}

/// Input for `GetTopicAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetTopicAttributesInput {
    /// The topic ARN.
    pub topic_arn: String,
}

/// Input for `SetTopicAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetTopicAttributesInput {
    /// The topic ARN.
    pub topic_arn: String,
    /// The attribute name to set.
    pub attribute_name: String,
    /// The attribute value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_value: Option<String>,
}

/// Input for `ListTopics`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListTopicsInput {
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Subscription management
// ---------------------------------------------------------------------------

/// Input for `Subscribe`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SubscribeInput {
    /// The topic ARN.
    pub topic_arn: String,
    /// The subscription protocol (e.g., `http`, `https`, `email`, `sqs`, `lambda`).
    pub protocol: String,
    /// The subscription endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// Subscription attributes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
    /// Whether to return the subscription ARN immediately.
    #[serde(default)]
    pub return_subscription_arn: bool,
}

/// Input for `Unsubscribe`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UnsubscribeInput {
    /// The subscription ARN.
    pub subscription_arn: String,
}

/// Input for `ConfirmSubscription`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConfirmSubscriptionInput {
    /// The topic ARN.
    pub topic_arn: String,
    /// The confirmation token.
    pub token: String,
    /// Whether to require authentication on unsubscribe.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticate_on_unsubscribe: Option<String>,
}

/// Input for `GetSubscriptionAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetSubscriptionAttributesInput {
    /// The subscription ARN.
    pub subscription_arn: String,
}

/// Input for `SetSubscriptionAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetSubscriptionAttributesInput {
    /// The subscription ARN.
    pub subscription_arn: String,
    /// The attribute name to set.
    pub attribute_name: String,
    /// The attribute value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_value: Option<String>,
}

/// Input for `ListSubscriptions`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListSubscriptionsInput {
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// Input for `ListSubscriptionsByTopic`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListSubscriptionsByTopicInput {
    /// The topic ARN.
    pub topic_arn: String,
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Publishing
// ---------------------------------------------------------------------------

/// Input for `Publish`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PublishInput {
    /// The topic ARN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_arn: Option<String>,
    /// The target ARN (for direct publishing to a platform endpoint).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_arn: Option<String>,
    /// The phone number to send an SMS to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
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

/// Input for `PublishBatch`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PublishBatchInput {
    /// The topic ARN.
    pub topic_arn: String,
    /// The batch entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub publish_batch_request_entries: Vec<PublishBatchRequestEntry>,
}

// ---------------------------------------------------------------------------
// Permissions
// ---------------------------------------------------------------------------

/// Input for `AddPermission`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AddPermissionInput {
    /// The topic ARN.
    pub topic_arn: String,
    /// A unique identifier for the permission statement.
    pub label: String,
    /// The AWS account IDs to grant permission to.
    #[serde(
        rename = "AWSAccountId",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub aws_account_id: Vec<String>,
    /// The action names to allow.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub action_name: Vec<String>,
}

/// Input for `RemovePermission`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RemovePermissionInput {
    /// The topic ARN.
    pub topic_arn: String,
    /// The permission statement label to remove.
    pub label: String,
}

// ---------------------------------------------------------------------------
// Tagging
// ---------------------------------------------------------------------------

/// Input for `TagResource`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TagResourceInput {
    /// The resource ARN.
    pub resource_arn: String,
    /// The tags to add.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
}

/// Input for `UntagResource`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UntagResourceInput {
    /// The resource ARN.
    pub resource_arn: String,
    /// The tag keys to remove.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tag_keys: Vec<String>,
}

/// Input for `ListTagsForResource`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListTagsForResourceInput {
    /// The resource ARN.
    pub resource_arn: String,
}

// ---------------------------------------------------------------------------
// Platform applications
// ---------------------------------------------------------------------------

/// Input for `CreatePlatformApplication`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreatePlatformApplicationInput {
    /// The application name.
    pub name: String,
    /// The platform (e.g., `APNS`, `GCM`).
    pub platform: String,
    /// Platform application attributes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

/// Input for `DeletePlatformApplication`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeletePlatformApplicationInput {
    /// The platform application ARN.
    pub platform_application_arn: String,
}

/// Input for `GetPlatformApplicationAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetPlatformApplicationAttributesInput {
    /// The platform application ARN.
    pub platform_application_arn: String,
}

/// Input for `SetPlatformApplicationAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetPlatformApplicationAttributesInput {
    /// The platform application ARN.
    pub platform_application_arn: String,
    /// Attributes to set.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

/// Input for `ListPlatformApplications`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListPlatformApplicationsInput {
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// Input for `CreatePlatformEndpoint`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreatePlatformEndpointInput {
    /// The platform application ARN.
    pub platform_application_arn: String,
    /// The device token.
    pub token: String,
    /// Custom user data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_user_data: Option<String>,
    /// Endpoint attributes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

/// Input for `DeleteEndpoint`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteEndpointInput {
    /// The endpoint ARN.
    pub endpoint_arn: String,
}

/// Input for `GetEndpointAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetEndpointAttributesInput {
    /// The endpoint ARN.
    pub endpoint_arn: String,
}

/// Input for `SetEndpointAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetEndpointAttributesInput {
    /// The endpoint ARN.
    pub endpoint_arn: String,
    /// Attributes to set.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

/// Input for `ListEndpointsByPlatformApplication`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListEndpointsByPlatformApplicationInput {
    /// The platform application ARN.
    pub platform_application_arn: String,
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

// ---------------------------------------------------------------------------
// SMS
// ---------------------------------------------------------------------------

/// Input for `CheckIfPhoneNumberIsOptedOut`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CheckIfPhoneNumberIsOptedOutInput {
    /// The phone number to check.
    pub phone_number: String,
}

/// Input for `GetSMSAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetSMSAttributesInput {
    /// The attribute names to retrieve. Empty means all.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<String>,
}

/// Input for `SetSMSAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetSMSAttributesInput {
    /// The attributes to set.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

/// Input for `ListPhoneNumbersOptedOut`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListPhoneNumbersOptedOutInput {
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// Input for `OptInPhoneNumber`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OptInPhoneNumberInput {
    /// The phone number to opt in.
    pub phone_number: String,
}

/// Input for `GetSMSSandboxAccountStatus`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetSMSSandboxAccountStatusInput {}

/// Input for `CreateSMSSandboxPhoneNumber`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateSMSSandboxPhoneNumberInput {
    /// The phone number to add.
    pub phone_number: String,
    /// The language code for the verification message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

/// Input for `DeleteSMSSandboxPhoneNumber`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteSMSSandboxPhoneNumberInput {
    /// The phone number to delete.
    pub phone_number: String,
}

/// Input for `VerifySMSSandboxPhoneNumber`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VerifySMSSandboxPhoneNumberInput {
    /// The phone number to verify.
    pub phone_number: String,
    /// The one-time password sent to the phone number.
    pub one_time_password: String,
}

/// Input for `ListSMSSandboxPhoneNumbers`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListSMSSandboxPhoneNumbersInput {
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// Input for `ListOriginationNumbers`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListOriginationNumbersInput {
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,
}

// ---------------------------------------------------------------------------
// Data protection
// ---------------------------------------------------------------------------

/// Input for `GetDataProtectionPolicy`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetDataProtectionPolicyInput {
    /// The resource ARN.
    pub resource_arn: String,
}

/// Input for `PutDataProtectionPolicy`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PutDataProtectionPolicyInput {
    /// The resource ARN.
    pub resource_arn: String,
    /// The data protection policy JSON.
    pub data_protection_policy: String,
}
