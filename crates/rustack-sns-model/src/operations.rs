//! SNS operation enum.

use std::fmt;

/// All supported SNS operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SnsOperation {
    // Topic management
    /// Create a new topic.
    CreateTopic,
    /// Delete a topic.
    DeleteTopic,
    /// Get topic attributes.
    GetTopicAttributes,
    /// Set topic attributes.
    SetTopicAttributes,
    /// List topics.
    ListTopics,

    // Subscription management
    /// Subscribe to a topic.
    Subscribe,
    /// Unsubscribe from a topic.
    Unsubscribe,
    /// Confirm a subscription.
    ConfirmSubscription,
    /// Get subscription attributes.
    GetSubscriptionAttributes,
    /// Set subscription attributes.
    SetSubscriptionAttributes,
    /// List subscriptions.
    ListSubscriptions,
    /// List subscriptions by topic.
    ListSubscriptionsByTopic,

    // Publishing
    /// Publish a message.
    Publish,
    /// Publish a batch of messages.
    PublishBatch,

    // Permissions
    /// Add a permission to a topic.
    AddPermission,
    /// Remove a permission from a topic.
    RemovePermission,

    // Tagging
    /// Add tags to a resource.
    TagResource,
    /// Remove tags from a resource.
    UntagResource,
    /// List tags for a resource.
    ListTagsForResource,

    // Platform applications
    /// Create a platform application.
    CreatePlatformApplication,
    /// Delete a platform application.
    DeletePlatformApplication,
    /// Get platform application attributes.
    GetPlatformApplicationAttributes,
    /// Set platform application attributes.
    SetPlatformApplicationAttributes,
    /// List platform applications.
    ListPlatformApplications,
    /// Create a platform endpoint.
    CreatePlatformEndpoint,
    /// Delete an endpoint.
    DeleteEndpoint,
    /// Get endpoint attributes.
    GetEndpointAttributes,
    /// Set endpoint attributes.
    SetEndpointAttributes,
    /// List endpoints by platform application.
    ListEndpointsByPlatformApplication,

    // SMS
    /// Check if a phone number is opted out.
    CheckIfPhoneNumberIsOptedOut,
    /// Get SMS attributes.
    GetSMSAttributes,
    /// Set SMS attributes.
    SetSMSAttributes,
    /// List phone numbers that have opted out.
    ListPhoneNumbersOptedOut,
    /// Opt in a phone number.
    OptInPhoneNumber,
    /// Get SMS sandbox account status.
    GetSMSSandboxAccountStatus,
    /// Create an SMS sandbox phone number.
    CreateSMSSandboxPhoneNumber,
    /// Delete an SMS sandbox phone number.
    DeleteSMSSandboxPhoneNumber,
    /// Verify an SMS sandbox phone number.
    VerifySMSSandboxPhoneNumber,
    /// List SMS sandbox phone numbers.
    ListSMSSandboxPhoneNumbers,
    /// List origination numbers.
    ListOriginationNumbers,

    // Data protection
    /// Get data protection policy.
    GetDataProtectionPolicy,
    /// Put data protection policy.
    PutDataProtectionPolicy,
}

impl SnsOperation {
    /// Returns the AWS operation name string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            // Topic management
            Self::CreateTopic => "CreateTopic",
            Self::DeleteTopic => "DeleteTopic",
            Self::GetTopicAttributes => "GetTopicAttributes",
            Self::SetTopicAttributes => "SetTopicAttributes",
            Self::ListTopics => "ListTopics",
            // Subscription management
            Self::Subscribe => "Subscribe",
            Self::Unsubscribe => "Unsubscribe",
            Self::ConfirmSubscription => "ConfirmSubscription",
            Self::GetSubscriptionAttributes => "GetSubscriptionAttributes",
            Self::SetSubscriptionAttributes => "SetSubscriptionAttributes",
            Self::ListSubscriptions => "ListSubscriptions",
            Self::ListSubscriptionsByTopic => "ListSubscriptionsByTopic",
            // Publishing
            Self::Publish => "Publish",
            Self::PublishBatch => "PublishBatch",
            // Permissions
            Self::AddPermission => "AddPermission",
            Self::RemovePermission => "RemovePermission",
            // Tagging
            Self::TagResource => "TagResource",
            Self::UntagResource => "UntagResource",
            Self::ListTagsForResource => "ListTagsForResource",
            // Platform applications
            Self::CreatePlatformApplication => "CreatePlatformApplication",
            Self::DeletePlatformApplication => "DeletePlatformApplication",
            Self::GetPlatformApplicationAttributes => "GetPlatformApplicationAttributes",
            Self::SetPlatformApplicationAttributes => "SetPlatformApplicationAttributes",
            Self::ListPlatformApplications => "ListPlatformApplications",
            Self::CreatePlatformEndpoint => "CreatePlatformEndpoint",
            Self::DeleteEndpoint => "DeleteEndpoint",
            Self::GetEndpointAttributes => "GetEndpointAttributes",
            Self::SetEndpointAttributes => "SetEndpointAttributes",
            Self::ListEndpointsByPlatformApplication => "ListEndpointsByPlatformApplication",
            // SMS
            Self::CheckIfPhoneNumberIsOptedOut => "CheckIfPhoneNumberIsOptedOut",
            Self::GetSMSAttributes => "GetSMSAttributes",
            Self::SetSMSAttributes => "SetSMSAttributes",
            Self::ListPhoneNumbersOptedOut => "ListPhoneNumbersOptedOut",
            Self::OptInPhoneNumber => "OptInPhoneNumber",
            Self::GetSMSSandboxAccountStatus => "GetSMSSandboxAccountStatus",
            Self::CreateSMSSandboxPhoneNumber => "CreateSMSSandboxPhoneNumber",
            Self::DeleteSMSSandboxPhoneNumber => "DeleteSMSSandboxPhoneNumber",
            Self::VerifySMSSandboxPhoneNumber => "VerifySMSSandboxPhoneNumber",
            Self::ListSMSSandboxPhoneNumbers => "ListSMSSandboxPhoneNumbers",
            Self::ListOriginationNumbers => "ListOriginationNumbers",
            // Data protection
            Self::GetDataProtectionPolicy => "GetDataProtectionPolicy",
            Self::PutDataProtectionPolicy => "PutDataProtectionPolicy",
        }
    }

    /// Parse an operation name string into an [`SnsOperation`].
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            // Topic management
            "CreateTopic" => Some(Self::CreateTopic),
            "DeleteTopic" => Some(Self::DeleteTopic),
            "GetTopicAttributes" => Some(Self::GetTopicAttributes),
            "SetTopicAttributes" => Some(Self::SetTopicAttributes),
            "ListTopics" => Some(Self::ListTopics),
            // Subscription management
            "Subscribe" => Some(Self::Subscribe),
            "Unsubscribe" => Some(Self::Unsubscribe),
            "ConfirmSubscription" => Some(Self::ConfirmSubscription),
            "GetSubscriptionAttributes" => Some(Self::GetSubscriptionAttributes),
            "SetSubscriptionAttributes" => Some(Self::SetSubscriptionAttributes),
            "ListSubscriptions" => Some(Self::ListSubscriptions),
            "ListSubscriptionsByTopic" => Some(Self::ListSubscriptionsByTopic),
            // Publishing
            "Publish" => Some(Self::Publish),
            "PublishBatch" => Some(Self::PublishBatch),
            // Permissions
            "AddPermission" => Some(Self::AddPermission),
            "RemovePermission" => Some(Self::RemovePermission),
            // Tagging
            "TagResource" => Some(Self::TagResource),
            "UntagResource" => Some(Self::UntagResource),
            "ListTagsForResource" => Some(Self::ListTagsForResource),
            // Platform applications
            "CreatePlatformApplication" => Some(Self::CreatePlatformApplication),
            "DeletePlatformApplication" => Some(Self::DeletePlatformApplication),
            "GetPlatformApplicationAttributes" => Some(Self::GetPlatformApplicationAttributes),
            "SetPlatformApplicationAttributes" => Some(Self::SetPlatformApplicationAttributes),
            "ListPlatformApplications" => Some(Self::ListPlatformApplications),
            "CreatePlatformEndpoint" => Some(Self::CreatePlatformEndpoint),
            "DeleteEndpoint" => Some(Self::DeleteEndpoint),
            "GetEndpointAttributes" => Some(Self::GetEndpointAttributes),
            "SetEndpointAttributes" => Some(Self::SetEndpointAttributes),
            "ListEndpointsByPlatformApplication" => Some(Self::ListEndpointsByPlatformApplication),
            // SMS
            "CheckIfPhoneNumberIsOptedOut" => Some(Self::CheckIfPhoneNumberIsOptedOut),
            "GetSMSAttributes" => Some(Self::GetSMSAttributes),
            "SetSMSAttributes" => Some(Self::SetSMSAttributes),
            "ListPhoneNumbersOptedOut" => Some(Self::ListPhoneNumbersOptedOut),
            "OptInPhoneNumber" => Some(Self::OptInPhoneNumber),
            "GetSMSSandboxAccountStatus" => Some(Self::GetSMSSandboxAccountStatus),
            "CreateSMSSandboxPhoneNumber" => Some(Self::CreateSMSSandboxPhoneNumber),
            "DeleteSMSSandboxPhoneNumber" => Some(Self::DeleteSMSSandboxPhoneNumber),
            "VerifySMSSandboxPhoneNumber" => Some(Self::VerifySMSSandboxPhoneNumber),
            "ListSMSSandboxPhoneNumbers" => Some(Self::ListSMSSandboxPhoneNumbers),
            "ListOriginationNumbers" => Some(Self::ListOriginationNumbers),
            // Data protection
            "GetDataProtectionPolicy" => Some(Self::GetDataProtectionPolicy),
            "PutDataProtectionPolicy" => Some(Self::PutDataProtectionPolicy),
            _ => None,
        }
    }
}

impl fmt::Display for SnsOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
