//! SNS operation output types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::{
    BatchResultErrorEntry, Endpoint, PhoneNumberInformation, PlatformApplication,
    PublishBatchResultEntry, SMSSandboxPhoneNumber, Subscription, Tag, Topic,
};

// ---------------------------------------------------------------------------
// Topic management
// ---------------------------------------------------------------------------

/// Output for `CreateTopic`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateTopicOutput {
    /// The topic ARN.
    pub topic_arn: String,
}

/// Output for `DeleteTopic` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeleteTopicOutput {}

/// Output for `GetTopicAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetTopicAttributesOutput {
    /// The topic attributes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

/// Output for `SetTopicAttributes` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SetTopicAttributesOutput {}

/// Output for `ListTopics`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListTopicsOutput {
    /// The topics.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub topics: Vec<Topic>,
    /// Pagination token for next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Subscription management
// ---------------------------------------------------------------------------

/// Output for `Subscribe`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SubscribeOutput {
    /// The subscription ARN (may be `pending confirmation`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_arn: Option<String>,
}

/// Output for `Unsubscribe` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnsubscribeOutput {}

/// Output for `ConfirmSubscription`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConfirmSubscriptionOutput {
    /// The subscription ARN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_arn: Option<String>,
}

/// Output for `GetSubscriptionAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetSubscriptionAttributesOutput {
    /// The subscription attributes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

/// Output for `SetSubscriptionAttributes` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SetSubscriptionAttributesOutput {}

/// Output for `ListSubscriptions`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListSubscriptionsOutput {
    /// The subscriptions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subscriptions: Vec<Subscription>,
    /// Pagination token for next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// Output for `ListSubscriptionsByTopic`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListSubscriptionsByTopicOutput {
    /// The subscriptions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subscriptions: Vec<Subscription>,
    /// Pagination token for next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Publishing
// ---------------------------------------------------------------------------

/// Output for `Publish`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PublishOutput {
    /// The message ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// The sequence number (FIFO topics only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence_number: Option<String>,
}

/// Output for `PublishBatch`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PublishBatchOutput {
    /// Successfully published entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub successful: Vec<PublishBatchResultEntry>,
    /// Failed entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failed: Vec<BatchResultErrorEntry>,
}

// ---------------------------------------------------------------------------
// Permissions
// ---------------------------------------------------------------------------

/// Output for `AddPermission` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AddPermissionOutput {}

/// Output for `RemovePermission` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RemovePermissionOutput {}

// ---------------------------------------------------------------------------
// Tagging
// ---------------------------------------------------------------------------

/// Output for `TagResource` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TagResourceOutput {}

/// Output for `UntagResource` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UntagResourceOutput {}

/// Output for `ListTagsForResource`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListTagsForResourceOutput {
    /// The tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
}

// ---------------------------------------------------------------------------
// Platform applications
// ---------------------------------------------------------------------------

/// Output for `CreatePlatformApplication`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreatePlatformApplicationOutput {
    /// The platform application ARN.
    pub platform_application_arn: String,
}

/// Output for `DeletePlatformApplication` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeletePlatformApplicationOutput {}

/// Output for `GetPlatformApplicationAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetPlatformApplicationAttributesOutput {
    /// The platform application attributes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

/// Output for `SetPlatformApplicationAttributes` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SetPlatformApplicationAttributesOutput {}

/// Output for `ListPlatformApplications`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListPlatformApplicationsOutput {
    /// The platform applications.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub platform_applications: Vec<PlatformApplication>,
    /// Pagination token for next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// Output for `CreatePlatformEndpoint`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreatePlatformEndpointOutput {
    /// The endpoint ARN.
    pub endpoint_arn: String,
}

/// Output for `DeleteEndpoint` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeleteEndpointOutput {}

/// Output for `GetEndpointAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetEndpointAttributesOutput {
    /// The endpoint attributes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

/// Output for `SetEndpointAttributes` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SetEndpointAttributesOutput {}

/// Output for `ListEndpointsByPlatformApplication`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListEndpointsByPlatformApplicationOutput {
    /// The endpoints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub endpoints: Vec<Endpoint>,
    /// Pagination token for next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

// ---------------------------------------------------------------------------
// SMS
// ---------------------------------------------------------------------------

/// Output for `CheckIfPhoneNumberIsOptedOut`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CheckIfPhoneNumberIsOptedOutOutput {
    /// Whether the phone number is opted out.
    #[serde(default)]
    pub is_opted_out: bool,
}

/// Output for `GetSMSAttributes`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetSMSAttributesOutput {
    /// The SMS attributes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

/// Output for `SetSMSAttributes` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SetSMSAttributesOutput {}

/// Output for `ListPhoneNumbersOptedOut`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListPhoneNumbersOptedOutOutput {
    /// The phone numbers that have opted out.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phone_numbers: Vec<String>,
    /// Pagination token for next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// Output for `OptInPhoneNumber` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptInPhoneNumberOutput {}

/// Output for `GetSMSSandboxAccountStatus`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetSMSSandboxAccountStatusOutput {
    /// Whether the account is in the SMS sandbox.
    #[serde(default)]
    pub is_in_sandbox: bool,
}

/// Output for `CreateSMSSandboxPhoneNumber` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateSMSSandboxPhoneNumberOutput {}

/// Output for `DeleteSMSSandboxPhoneNumber` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeleteSMSSandboxPhoneNumberOutput {}

/// Output for `VerifySMSSandboxPhoneNumber` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerifySMSSandboxPhoneNumberOutput {}

/// Output for `ListSMSSandboxPhoneNumbers`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListSMSSandboxPhoneNumbersOutput {
    /// The SMS sandbox phone numbers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phone_numbers: Vec<SMSSandboxPhoneNumber>,
    /// Pagination token for next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// Output for `ListOriginationNumbers`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListOriginationNumbersOutput {
    /// The origination phone numbers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phone_numbers: Vec<PhoneNumberInformation>,
    /// Pagination token for next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Data protection
// ---------------------------------------------------------------------------

/// Output for `GetDataProtectionPolicy`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetDataProtectionPolicyOutput {
    /// The data protection policy JSON.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_protection_policy: Option<String>,
}

/// Output for `PutDataProtectionPolicy` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PutDataProtectionPolicyOutput {}
