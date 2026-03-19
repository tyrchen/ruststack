//! Auto-generated from AWS SES Smithy model. DO NOT EDIT.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::{
    ConfigurationSet, DeliveryOptions, EventDestination, IdentityDkimAttributes,
    IdentityMailFromDomainAttributes, IdentityNotificationAttributes,
    IdentityVerificationAttributes, ReceiptRule, ReceiptRuleSetMetadata, ReputationOptions,
    SendDataPoint, Template, TemplateMetadata, TrackingOptions,
};

/// SES CloneReceiptRuleSetResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CloneReceiptRuleSetResponse {}

/// SES CreateConfigurationSetEventDestinationResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateConfigurationSetEventDestinationResponse {}

/// SES CreateConfigurationSetResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateConfigurationSetResponse {}

/// SES CreateReceiptRuleResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateReceiptRuleResponse {}

/// SES CreateReceiptRuleSetResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateReceiptRuleSetResponse {}

/// SES CreateTemplateResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateTemplateResponse {}

/// SES DeleteConfigurationSetEventDestinationResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteConfigurationSetEventDestinationResponse {}

/// SES DeleteConfigurationSetResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteConfigurationSetResponse {}

/// SES DeleteIdentityPolicyResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteIdentityPolicyResponse {}

/// SES DeleteIdentityResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteIdentityResponse {}

/// SES DeleteReceiptRuleResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteReceiptRuleResponse {}

/// SES DeleteReceiptRuleSetResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteReceiptRuleSetResponse {}

/// SES DeleteTemplateResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteTemplateResponse {}

/// SES DescribeActiveReceiptRuleSetResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribeActiveReceiptRuleSetResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ReceiptRuleSetMetadata>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<ReceiptRule>,
}

/// SES DescribeConfigurationSetResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribeConfigurationSetResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_set: Option<ConfigurationSet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_options: Option<DeliveryOptions>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event_destinations: Vec<EventDestination>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reputation_options: Option<ReputationOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_options: Option<TrackingOptions>,
}

/// SES DescribeReceiptRuleSetResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribeReceiptRuleSetResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ReceiptRuleSetMetadata>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<ReceiptRule>,
}

/// SES GetIdentityDkimAttributesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetIdentityDkimAttributesResponse {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dkim_attributes: HashMap<String, IdentityDkimAttributes>,
}

/// SES GetIdentityMailFromDomainAttributesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetIdentityMailFromDomainAttributesResponse {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub mail_from_domain_attributes: HashMap<String, IdentityMailFromDomainAttributes>,
}

/// SES GetIdentityNotificationAttributesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetIdentityNotificationAttributesResponse {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub notification_attributes: HashMap<String, IdentityNotificationAttributes>,
}

/// SES GetIdentityPoliciesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetIdentityPoliciesResponse {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub policies: HashMap<String, String>,
}

/// SES GetIdentityVerificationAttributesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetIdentityVerificationAttributesResponse {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub verification_attributes: HashMap<String, IdentityVerificationAttributes>,
}

/// SES GetSendQuotaResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetSendQuotaResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max24_hour_send: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_send_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sent_last24_hours: Option<f64>,
}

/// SES GetSendStatisticsResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetSendStatisticsResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub send_data_points: Vec<SendDataPoint>,
}

/// SES GetTemplateResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetTemplateResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<Template>,
}

/// SES ListConfigurationSetsResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListConfigurationSetsResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub configuration_sets: Vec<ConfigurationSet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// SES ListIdentitiesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListIdentitiesResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// SES ListIdentityPoliciesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListIdentityPoliciesResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_names: Vec<String>,
}

/// SES ListTemplatesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListTemplatesResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub templates_metadata: Vec<TemplateMetadata>,
}

/// SES ListVerifiedEmailAddressesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListVerifiedEmailAddressesResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verified_email_addresses: Vec<String>,
}

/// SES PutIdentityPolicyResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PutIdentityPolicyResponse {}

/// SES SendEmailResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendEmailResponse {
    pub message_id: String,
}

/// SES SendRawEmailResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendRawEmailResponse {
    pub message_id: String,
}

/// SES SendTemplatedEmailResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendTemplatedEmailResponse {
    pub message_id: String,
}

/// SES SetActiveReceiptRuleSetResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetActiveReceiptRuleSetResponse {}

/// SES SetIdentityFeedbackForwardingEnabledResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetIdentityFeedbackForwardingEnabledResponse {}

/// SES SetIdentityMailFromDomainResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetIdentityMailFromDomainResponse {}

/// SES SetIdentityNotificationTopicResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetIdentityNotificationTopicResponse {}

/// SES UpdateConfigurationSetEventDestinationResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateConfigurationSetEventDestinationResponse {}

/// SES UpdateTemplateResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateTemplateResponse {}

/// SES VerifyDomainDkimResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VerifyDomainDkimResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dkim_tokens: Vec<String>,
}

/// SES VerifyDomainIdentityResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VerifyDomainIdentityResponse {
    pub verification_token: String,
}

/// SES VerifyEmailIdentityResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VerifyEmailIdentityResponse {}
