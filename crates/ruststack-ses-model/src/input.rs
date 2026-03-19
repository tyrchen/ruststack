//! Auto-generated from AWS SES Smithy model. DO NOT EDIT.

use serde::{Deserialize, Serialize};

use crate::types::{
    BehaviorOnMXFailure, ConfigurationSet, ConfigurationSetAttribute, Destination,
    EventDestination, IdentityType, Message, MessageTag, NotificationType, RawMessage, ReceiptRule,
    Template,
};

/// SES CloneReceiptRuleSetInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CloneReceiptRuleSetInput {
    pub original_rule_set_name: String,
    pub rule_set_name: String,
}

/// SES CreateConfigurationSetEventDestinationInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateConfigurationSetEventDestinationInput {
    pub configuration_set_name: String,
    pub event_destination: EventDestination,
}

/// SES CreateConfigurationSetInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateConfigurationSetInput {
    pub configuration_set: ConfigurationSet,
}

/// SES CreateReceiptRuleInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateReceiptRuleInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    pub rule: ReceiptRule,
    pub rule_set_name: String,
}

/// SES CreateReceiptRuleSetInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateReceiptRuleSetInput {
    pub rule_set_name: String,
}

/// SES CreateTemplateInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateTemplateInput {
    pub template: Template,
}

/// SES DeleteConfigurationSetEventDestinationInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteConfigurationSetEventDestinationInput {
    pub configuration_set_name: String,
    pub event_destination_name: String,
}

/// SES DeleteConfigurationSetInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteConfigurationSetInput {
    pub configuration_set_name: String,
}

/// SES DeleteIdentityInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteIdentityInput {
    pub identity: String,
}

/// SES DeleteIdentityPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteIdentityPolicyInput {
    pub identity: String,
    pub policy_name: String,
}

/// SES DeleteReceiptRuleInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteReceiptRuleInput {
    pub rule_name: String,
    pub rule_set_name: String,
}

/// SES DeleteReceiptRuleSetInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteReceiptRuleSetInput {
    pub rule_set_name: String,
}

/// SES DeleteTemplateInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteTemplateInput {
    pub template_name: String,
}

/// SES DeleteVerifiedEmailAddressInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteVerifiedEmailAddressInput {
    pub email_address: String,
}

/// SES DescribeActiveReceiptRuleSetInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribeActiveReceiptRuleSetInput {}

/// SES DescribeConfigurationSetInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribeConfigurationSetInput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub configuration_set_attribute_names: Vec<ConfigurationSetAttribute>,
    pub configuration_set_name: String,
}

/// SES DescribeReceiptRuleSetInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribeReceiptRuleSetInput {
    pub rule_set_name: String,
}

/// SES GetIdentityDkimAttributesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetIdentityDkimAttributesInput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identities: Vec<String>,
}

/// SES GetIdentityMailFromDomainAttributesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetIdentityMailFromDomainAttributesInput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identities: Vec<String>,
}

/// SES GetIdentityNotificationAttributesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetIdentityNotificationAttributesInput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identities: Vec<String>,
}

/// SES GetIdentityPoliciesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetIdentityPoliciesInput {
    pub identity: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_names: Vec<String>,
}

/// SES GetIdentityVerificationAttributesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetIdentityVerificationAttributesInput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identities: Vec<String>,
}

/// SES GetTemplateInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetTemplateInput {
    pub template_name: String,
}

/// SES ListConfigurationSetsInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListConfigurationSetsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// SES ListIdentitiesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListIdentitiesInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_type: Option<IdentityType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// SES ListIdentityPoliciesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListIdentityPoliciesInput {
    pub identity: String,
}

/// SES ListTemplatesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListTemplatesInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// SES PutIdentityPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PutIdentityPolicyInput {
    pub identity: String,
    pub policy: String,
    pub policy_name: String,
}

/// SES SendEmailInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendEmailInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_set_name: Option<String>,
    pub destination: Destination,
    pub message: Message,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reply_to_addresses: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_path_arn: Option<String>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_arn: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<MessageTag>,
}

/// SES SendRawEmailInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendRawEmailInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_set_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub destinations: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_arn: Option<String>,
    pub raw_message: RawMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_path_arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_arn: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<MessageTag>,
}

/// SES SendTemplatedEmailInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendTemplatedEmailInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_set_name: Option<String>,
    pub destination: Destination,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reply_to_addresses: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_path_arn: Option<String>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_arn: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<MessageTag>,
    pub template: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_arn: Option<String>,
    pub template_data: String,
}

/// SES SetActiveReceiptRuleSetInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetActiveReceiptRuleSetInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_set_name: Option<String>,
}

/// SES SetIdentityFeedbackForwardingEnabledInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetIdentityFeedbackForwardingEnabledInput {
    pub forwarding_enabled: bool,
    pub identity: String,
}

/// SES SetIdentityMailFromDomainInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetIdentityMailFromDomainInput {
    #[serde(rename = "BehaviorOnMXFailure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior_on_mx_failure: Option<BehaviorOnMXFailure>,
    pub identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mail_from_domain: Option<String>,
}

/// SES SetIdentityNotificationTopicInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetIdentityNotificationTopicInput {
    pub identity: String,
    pub notification_type: NotificationType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sns_topic: Option<String>,
}

/// SES UpdateConfigurationSetEventDestinationInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateConfigurationSetEventDestinationInput {
    pub configuration_set_name: String,
    pub event_destination: EventDestination,
}

/// SES UpdateTemplateInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateTemplateInput {
    pub template: Template,
}

/// SES VerifyDomainDkimInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VerifyDomainDkimInput {
    pub domain: String,
}

/// SES VerifyDomainIdentityInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VerifyDomainIdentityInput {
    pub domain: String,
}

/// SES VerifyEmailAddressInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VerifyEmailAddressInput {
    pub email_address: String,
}

/// SES VerifyEmailIdentityInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VerifyEmailIdentityInput {
    pub email_address: String,
}
