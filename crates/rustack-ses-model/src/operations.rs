//! Auto-generated from AWS SES Smithy model. DO NOT EDIT.

/// All supported Ses operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SesOperation {
    /// The VerifyEmailIdentity operation.
    VerifyEmailIdentity,
    /// The VerifyDomainIdentity operation.
    VerifyDomainIdentity,
    /// The ListIdentities operation.
    ListIdentities,
    /// The DeleteIdentity operation.
    DeleteIdentity,
    /// The GetIdentityVerificationAttributes operation.
    GetIdentityVerificationAttributes,
    /// The VerifyEmailAddress operation.
    VerifyEmailAddress,
    /// The DeleteVerifiedEmailAddress operation.
    DeleteVerifiedEmailAddress,
    /// The ListVerifiedEmailAddresses operation.
    ListVerifiedEmailAddresses,
    /// The SendEmail operation.
    SendEmail,
    /// The SendRawEmail operation.
    SendRawEmail,
    /// The GetSendQuota operation.
    GetSendQuota,
    /// The GetSendStatistics operation.
    GetSendStatistics,
    /// The CreateTemplate operation.
    CreateTemplate,
    /// The GetTemplate operation.
    GetTemplate,
    /// The UpdateTemplate operation.
    UpdateTemplate,
    /// The DeleteTemplate operation.
    DeleteTemplate,
    /// The ListTemplates operation.
    ListTemplates,
    /// The SendTemplatedEmail operation.
    SendTemplatedEmail,
    /// The CreateConfigurationSet operation.
    CreateConfigurationSet,
    /// The DeleteConfigurationSet operation.
    DeleteConfigurationSet,
    /// The DescribeConfigurationSet operation.
    DescribeConfigurationSet,
    /// The ListConfigurationSets operation.
    ListConfigurationSets,
    /// The CreateConfigurationSetEventDestination operation.
    CreateConfigurationSetEventDestination,
    /// The UpdateConfigurationSetEventDestination operation.
    UpdateConfigurationSetEventDestination,
    /// The DeleteConfigurationSetEventDestination operation.
    DeleteConfigurationSetEventDestination,
    /// The CreateReceiptRuleSet operation.
    CreateReceiptRuleSet,
    /// The DeleteReceiptRuleSet operation.
    DeleteReceiptRuleSet,
    /// The CreateReceiptRule operation.
    CreateReceiptRule,
    /// The DeleteReceiptRule operation.
    DeleteReceiptRule,
    /// The DescribeReceiptRuleSet operation.
    DescribeReceiptRuleSet,
    /// The CloneReceiptRuleSet operation.
    CloneReceiptRuleSet,
    /// The DescribeActiveReceiptRuleSet operation.
    DescribeActiveReceiptRuleSet,
    /// The SetActiveReceiptRuleSet operation.
    SetActiveReceiptRuleSet,
    /// The SetIdentityNotificationTopic operation.
    SetIdentityNotificationTopic,
    /// The SetIdentityFeedbackForwardingEnabled operation.
    SetIdentityFeedbackForwardingEnabled,
    /// The GetIdentityNotificationAttributes operation.
    GetIdentityNotificationAttributes,
    /// The VerifyDomainDkim operation.
    VerifyDomainDkim,
    /// The GetIdentityDkimAttributes operation.
    GetIdentityDkimAttributes,
    /// The SetIdentityMailFromDomain operation.
    SetIdentityMailFromDomain,
    /// The GetIdentityMailFromDomainAttributes operation.
    GetIdentityMailFromDomainAttributes,
    /// The GetIdentityPolicies operation.
    GetIdentityPolicies,
    /// The PutIdentityPolicy operation.
    PutIdentityPolicy,
    /// The DeleteIdentityPolicy operation.
    DeleteIdentityPolicy,
    /// The ListIdentityPolicies operation.
    ListIdentityPolicies,
}

impl SesOperation {
    /// Returns the AWS operation name string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VerifyEmailIdentity => "VerifyEmailIdentity",
            Self::VerifyDomainIdentity => "VerifyDomainIdentity",
            Self::ListIdentities => "ListIdentities",
            Self::DeleteIdentity => "DeleteIdentity",
            Self::GetIdentityVerificationAttributes => "GetIdentityVerificationAttributes",
            Self::VerifyEmailAddress => "VerifyEmailAddress",
            Self::DeleteVerifiedEmailAddress => "DeleteVerifiedEmailAddress",
            Self::ListVerifiedEmailAddresses => "ListVerifiedEmailAddresses",
            Self::SendEmail => "SendEmail",
            Self::SendRawEmail => "SendRawEmail",
            Self::GetSendQuota => "GetSendQuota",
            Self::GetSendStatistics => "GetSendStatistics",
            Self::CreateTemplate => "CreateTemplate",
            Self::GetTemplate => "GetTemplate",
            Self::UpdateTemplate => "UpdateTemplate",
            Self::DeleteTemplate => "DeleteTemplate",
            Self::ListTemplates => "ListTemplates",
            Self::SendTemplatedEmail => "SendTemplatedEmail",
            Self::CreateConfigurationSet => "CreateConfigurationSet",
            Self::DeleteConfigurationSet => "DeleteConfigurationSet",
            Self::DescribeConfigurationSet => "DescribeConfigurationSet",
            Self::ListConfigurationSets => "ListConfigurationSets",
            Self::CreateConfigurationSetEventDestination => {
                "CreateConfigurationSetEventDestination"
            }
            Self::UpdateConfigurationSetEventDestination => {
                "UpdateConfigurationSetEventDestination"
            }
            Self::DeleteConfigurationSetEventDestination => {
                "DeleteConfigurationSetEventDestination"
            }
            Self::CreateReceiptRuleSet => "CreateReceiptRuleSet",
            Self::DeleteReceiptRuleSet => "DeleteReceiptRuleSet",
            Self::CreateReceiptRule => "CreateReceiptRule",
            Self::DeleteReceiptRule => "DeleteReceiptRule",
            Self::DescribeReceiptRuleSet => "DescribeReceiptRuleSet",
            Self::CloneReceiptRuleSet => "CloneReceiptRuleSet",
            Self::DescribeActiveReceiptRuleSet => "DescribeActiveReceiptRuleSet",
            Self::SetActiveReceiptRuleSet => "SetActiveReceiptRuleSet",
            Self::SetIdentityNotificationTopic => "SetIdentityNotificationTopic",
            Self::SetIdentityFeedbackForwardingEnabled => "SetIdentityFeedbackForwardingEnabled",
            Self::GetIdentityNotificationAttributes => "GetIdentityNotificationAttributes",
            Self::VerifyDomainDkim => "VerifyDomainDkim",
            Self::GetIdentityDkimAttributes => "GetIdentityDkimAttributes",
            Self::SetIdentityMailFromDomain => "SetIdentityMailFromDomain",
            Self::GetIdentityMailFromDomainAttributes => "GetIdentityMailFromDomainAttributes",
            Self::GetIdentityPolicies => "GetIdentityPolicies",
            Self::PutIdentityPolicy => "PutIdentityPolicy",
            Self::DeleteIdentityPolicy => "DeleteIdentityPolicy",
            Self::ListIdentityPolicies => "ListIdentityPolicies",
        }
    }

    /// Parse an operation name string into an SesOperation.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "VerifyEmailIdentity" => Some(Self::VerifyEmailIdentity),
            "VerifyDomainIdentity" => Some(Self::VerifyDomainIdentity),
            "ListIdentities" => Some(Self::ListIdentities),
            "DeleteIdentity" => Some(Self::DeleteIdentity),
            "GetIdentityVerificationAttributes" => Some(Self::GetIdentityVerificationAttributes),
            "VerifyEmailAddress" => Some(Self::VerifyEmailAddress),
            "DeleteVerifiedEmailAddress" => Some(Self::DeleteVerifiedEmailAddress),
            "ListVerifiedEmailAddresses" => Some(Self::ListVerifiedEmailAddresses),
            "SendEmail" => Some(Self::SendEmail),
            "SendRawEmail" => Some(Self::SendRawEmail),
            "GetSendQuota" => Some(Self::GetSendQuota),
            "GetSendStatistics" => Some(Self::GetSendStatistics),
            "CreateTemplate" => Some(Self::CreateTemplate),
            "GetTemplate" => Some(Self::GetTemplate),
            "UpdateTemplate" => Some(Self::UpdateTemplate),
            "DeleteTemplate" => Some(Self::DeleteTemplate),
            "ListTemplates" => Some(Self::ListTemplates),
            "SendTemplatedEmail" => Some(Self::SendTemplatedEmail),
            "CreateConfigurationSet" => Some(Self::CreateConfigurationSet),
            "DeleteConfigurationSet" => Some(Self::DeleteConfigurationSet),
            "DescribeConfigurationSet" => Some(Self::DescribeConfigurationSet),
            "ListConfigurationSets" => Some(Self::ListConfigurationSets),
            "CreateConfigurationSetEventDestination" => {
                Some(Self::CreateConfigurationSetEventDestination)
            }
            "UpdateConfigurationSetEventDestination" => {
                Some(Self::UpdateConfigurationSetEventDestination)
            }
            "DeleteConfigurationSetEventDestination" => {
                Some(Self::DeleteConfigurationSetEventDestination)
            }
            "CreateReceiptRuleSet" => Some(Self::CreateReceiptRuleSet),
            "DeleteReceiptRuleSet" => Some(Self::DeleteReceiptRuleSet),
            "CreateReceiptRule" => Some(Self::CreateReceiptRule),
            "DeleteReceiptRule" => Some(Self::DeleteReceiptRule),
            "DescribeReceiptRuleSet" => Some(Self::DescribeReceiptRuleSet),
            "CloneReceiptRuleSet" => Some(Self::CloneReceiptRuleSet),
            "DescribeActiveReceiptRuleSet" => Some(Self::DescribeActiveReceiptRuleSet),
            "SetActiveReceiptRuleSet" => Some(Self::SetActiveReceiptRuleSet),
            "SetIdentityNotificationTopic" => Some(Self::SetIdentityNotificationTopic),
            "SetIdentityFeedbackForwardingEnabled" => {
                Some(Self::SetIdentityFeedbackForwardingEnabled)
            }
            "GetIdentityNotificationAttributes" => Some(Self::GetIdentityNotificationAttributes),
            "VerifyDomainDkim" => Some(Self::VerifyDomainDkim),
            "GetIdentityDkimAttributes" => Some(Self::GetIdentityDkimAttributes),
            "SetIdentityMailFromDomain" => Some(Self::SetIdentityMailFromDomain),
            "GetIdentityMailFromDomainAttributes" => {
                Some(Self::GetIdentityMailFromDomainAttributes)
            }
            "GetIdentityPolicies" => Some(Self::GetIdentityPolicies),
            "PutIdentityPolicy" => Some(Self::PutIdentityPolicy),
            "DeleteIdentityPolicy" => Some(Self::DeleteIdentityPolicy),
            "ListIdentityPolicies" => Some(Self::ListIdentityPolicies),
            _ => None,
        }
    }
}

impl std::fmt::Display for SesOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
