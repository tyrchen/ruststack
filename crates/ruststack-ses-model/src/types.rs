//! Auto-generated from AWS SES Smithy model. DO NOT EDIT.

use serde::{Deserialize, Serialize};

/// SES BehaviorOnMXFailure enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum BehaviorOnMXFailure {
    /// Default variant.
    #[default]
    RejectMessage,
    UseDefaultValue,
}

impl BehaviorOnMXFailure {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RejectMessage => "RejectMessage",
            Self::UseDefaultValue => "UseDefaultValue",
        }
    }
}

impl std::fmt::Display for BehaviorOnMXFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for BehaviorOnMXFailure {
    fn from(s: &str) -> Self {
        match s {
            "RejectMessage" => Self::RejectMessage,
            "UseDefaultValue" => Self::UseDefaultValue,
            _ => Self::default(),
        }
    }
}

/// SES ConfigurationSetAttribute enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ConfigurationSetAttribute {
    /// Default variant.
    #[default]
    #[serde(rename = "deliveryOptions")]
    DeliveryOptions,
    #[serde(rename = "eventDestinations")]
    EventDestinations,
    #[serde(rename = "reputationOptions")]
    ReputationOptions,
    #[serde(rename = "trackingOptions")]
    TrackingOptions,
}

impl ConfigurationSetAttribute {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DeliveryOptions => "deliveryOptions",
            Self::EventDestinations => "eventDestinations",
            Self::ReputationOptions => "reputationOptions",
            Self::TrackingOptions => "trackingOptions",
        }
    }
}

impl std::fmt::Display for ConfigurationSetAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for ConfigurationSetAttribute {
    fn from(s: &str) -> Self {
        match s {
            "deliveryOptions" => Self::DeliveryOptions,
            "eventDestinations" => Self::EventDestinations,
            "reputationOptions" => Self::ReputationOptions,
            "trackingOptions" => Self::TrackingOptions,
            _ => Self::default(),
        }
    }
}

/// SES CustomMailFromStatus enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum CustomMailFromStatus {
    /// Default variant.
    #[default]
    Failed,
    Pending,
    Success,
    TemporaryFailure,
}

impl CustomMailFromStatus {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Failed => "Failed",
            Self::Pending => "Pending",
            Self::Success => "Success",
            Self::TemporaryFailure => "TemporaryFailure",
        }
    }
}

impl std::fmt::Display for CustomMailFromStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for CustomMailFromStatus {
    fn from(s: &str) -> Self {
        match s {
            "Failed" => Self::Failed,
            "Pending" => Self::Pending,
            "Success" => Self::Success,
            "TemporaryFailure" => Self::TemporaryFailure,
            _ => Self::default(),
        }
    }
}

/// SES DimensionValueSource enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DimensionValueSource {
    /// Default variant.
    #[default]
    #[serde(rename = "emailHeader")]
    EmailHeader,
    #[serde(rename = "linkTag")]
    LinkTag,
    #[serde(rename = "messageTag")]
    MessageTag,
}

impl DimensionValueSource {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EmailHeader => "emailHeader",
            Self::LinkTag => "linkTag",
            Self::MessageTag => "messageTag",
        }
    }
}

impl std::fmt::Display for DimensionValueSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for DimensionValueSource {
    fn from(s: &str) -> Self {
        match s {
            "emailHeader" => Self::EmailHeader,
            "linkTag" => Self::LinkTag,
            "messageTag" => Self::MessageTag,
            _ => Self::default(),
        }
    }
}

/// SES EventType enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum EventType {
    /// Default variant.
    #[default]
    #[serde(rename = "bounce")]
    Bounce,
    #[serde(rename = "click")]
    Click,
    #[serde(rename = "complaint")]
    Complaint,
    #[serde(rename = "delivery")]
    Delivery,
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "reject")]
    Reject,
    #[serde(rename = "renderingFailure")]
    RenderingFailure,
    #[serde(rename = "send")]
    Send,
}

impl EventType {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bounce => "bounce",
            Self::Click => "click",
            Self::Complaint => "complaint",
            Self::Delivery => "delivery",
            Self::Open => "open",
            Self::Reject => "reject",
            Self::RenderingFailure => "renderingFailure",
            Self::Send => "send",
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for EventType {
    fn from(s: &str) -> Self {
        match s {
            "bounce" => Self::Bounce,
            "click" => Self::Click,
            "complaint" => Self::Complaint,
            "delivery" => Self::Delivery,
            "open" => Self::Open,
            "reject" => Self::Reject,
            "renderingFailure" => Self::RenderingFailure,
            "send" => Self::Send,
            _ => Self::default(),
        }
    }
}

/// SES IdentityType enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum IdentityType {
    /// Default variant.
    #[default]
    Domain,
    EmailAddress,
}

impl IdentityType {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Domain => "Domain",
            Self::EmailAddress => "EmailAddress",
        }
    }
}

impl std::fmt::Display for IdentityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for IdentityType {
    fn from(s: &str) -> Self {
        match s {
            "Domain" => Self::Domain,
            "EmailAddress" => Self::EmailAddress,
            _ => Self::default(),
        }
    }
}

/// SES InvocationType enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum InvocationType {
    /// Default variant.
    #[default]
    Event,
    RequestResponse,
}

impl InvocationType {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Event => "Event",
            Self::RequestResponse => "RequestResponse",
        }
    }
}

impl std::fmt::Display for InvocationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for InvocationType {
    fn from(s: &str) -> Self {
        match s {
            "Event" => Self::Event,
            "RequestResponse" => Self::RequestResponse,
            _ => Self::default(),
        }
    }
}

/// SES NotificationType enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum NotificationType {
    /// Default variant.
    #[default]
    Bounce,
    Complaint,
    Delivery,
}

impl NotificationType {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bounce => "Bounce",
            Self::Complaint => "Complaint",
            Self::Delivery => "Delivery",
        }
    }
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for NotificationType {
    fn from(s: &str) -> Self {
        match s {
            "Bounce" => Self::Bounce,
            "Complaint" => Self::Complaint,
            "Delivery" => Self::Delivery,
            _ => Self::default(),
        }
    }
}

/// SES SNSActionEncoding enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SNSActionEncoding {
    /// Default variant.
    #[default]
    Base64,
    #[serde(rename = "UTF-8")]
    Utf8,
}

impl SNSActionEncoding {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Base64 => "Base64",
            Self::Utf8 => "UTF-8",
        }
    }
}

impl std::fmt::Display for SNSActionEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for SNSActionEncoding {
    fn from(s: &str) -> Self {
        match s {
            "Base64" => Self::Base64,
            "UTF-8" => Self::Utf8,
            _ => Self::default(),
        }
    }
}

/// SES StopScope enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum StopScope {
    /// Default variant.
    #[default]
    RuleSet,
}

impl StopScope {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RuleSet => "RuleSet",
        }
    }
}

impl std::fmt::Display for StopScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for StopScope {
    fn from(s: &str) -> Self {
        match s {
            "RuleSet" => Self::RuleSet,
            _ => Self::default(),
        }
    }
}

/// SES TlsPolicy enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum TlsPolicy {
    /// Default variant.
    #[default]
    Optional,
    Require,
}

impl TlsPolicy {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Optional => "Optional",
            Self::Require => "Require",
        }
    }
}

impl std::fmt::Display for TlsPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for TlsPolicy {
    fn from(s: &str) -> Self {
        match s {
            "Optional" => Self::Optional,
            "Require" => Self::Require,
            _ => Self::default(),
        }
    }
}

/// SES VerificationStatus enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Default variant.
    #[default]
    Failed,
    NotStarted,
    Pending,
    Success,
    TemporaryFailure,
}

impl VerificationStatus {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Failed => "Failed",
            Self::NotStarted => "NotStarted",
            Self::Pending => "Pending",
            Self::Success => "Success",
            Self::TemporaryFailure => "TemporaryFailure",
        }
    }
}

impl std::fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for VerificationStatus {
    fn from(s: &str) -> Self {
        match s {
            "Failed" => Self::Failed,
            "NotStarted" => Self::NotStarted,
            "Pending" => Self::Pending,
            "Success" => Self::Success,
            "TemporaryFailure" => Self::TemporaryFailure,
            _ => Self::default(),
        }
    }
}

/// SES AddHeaderAction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AddHeaderAction {
    pub header_name: String,
    pub header_value: String,
}

/// SES Body.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Body {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Content>,
}

/// SES BounceAction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BounceAction {
    pub message: String,
    pub sender: String,
    pub smtp_reply_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_arn: Option<String>,
}

/// SES CloudWatchDestination.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CloudWatchDestination {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dimension_configurations: Vec<CloudWatchDimensionConfiguration>,
}

/// SES CloudWatchDimensionConfiguration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CloudWatchDimensionConfiguration {
    pub default_dimension_value: String,
    pub dimension_name: String,
    pub dimension_value_source: DimensionValueSource,
}

/// SES ConfigurationSet.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigurationSet {
    pub name: String,
}

/// SES ConnectAction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConnectAction {
    #[serde(rename = "IAMRoleARN")]
    pub iam_role_arn: String,
    #[serde(rename = "InstanceARN")]
    pub instance_arn: String,
}

/// SES Content.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Content {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charset: Option<String>,
    pub data: String,
}

/// SES DeliveryOptions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeliveryOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_policy: Option<TlsPolicy>,
}

/// SES Destination.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Destination {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bcc_addresses: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cc_addresses: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to_addresses: Vec<String>,
}

/// SES EventDestination.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EventDestination {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_watch_destination: Option<CloudWatchDestination>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kinesis_firehose_destination: Option<KinesisFirehoseDestination>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matching_event_types: Vec<EventType>,
    pub name: String,
    #[serde(rename = "SNSDestination")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sns_destination: Option<SNSDestination>,
}

/// SES IdentityDkimAttributes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IdentityDkimAttributes {
    pub dkim_enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dkim_tokens: Vec<String>,
    pub dkim_verification_status: VerificationStatus,
}

/// SES IdentityMailFromDomainAttributes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IdentityMailFromDomainAttributes {
    #[serde(rename = "BehaviorOnMXFailure")]
    pub behavior_on_mx_failure: BehaviorOnMXFailure,
    pub mail_from_domain: String,
    pub mail_from_domain_status: CustomMailFromStatus,
}

/// SES IdentityNotificationAttributes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IdentityNotificationAttributes {
    pub bounce_topic: String,
    pub complaint_topic: String,
    pub delivery_topic: String,
    pub forwarding_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers_in_bounce_notifications_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers_in_complaint_notifications_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers_in_delivery_notifications_enabled: Option<bool>,
}

/// SES IdentityVerificationAttributes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IdentityVerificationAttributes {
    pub verification_status: VerificationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_token: Option<String>,
}

/// SES KinesisFirehoseDestination.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KinesisFirehoseDestination {
    #[serde(rename = "DeliveryStreamARN")]
    pub delivery_stream_arn: String,
    #[serde(rename = "IAMRoleARN")]
    pub iam_role_arn: String,
}

/// SES LambdaAction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LambdaAction {
    pub function_arn: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invocation_type: Option<InvocationType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_arn: Option<String>,
}

/// SES Message.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Message {
    pub body: Body,
    pub subject: Content,
}

/// SES MessageTag.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MessageTag {
    pub name: String,
    pub value: String,
}

/// SES RawMessage.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawMessage {
    pub data: Vec<u8>,
}

/// SES ReceiptAction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReceiptAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_header_action: Option<AddHeaderAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounce_action: Option<BounceAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_action: Option<ConnectAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lambda_action: Option<LambdaAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s3_action: Option<S3Action>,
    #[serde(rename = "SNSAction")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sns_action: Option<SNSAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_action: Option<StopAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workmail_action: Option<WorkmailAction>,
}

/// SES ReceiptRule.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReceiptRule {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<ReceiptAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recipients: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_policy: Option<TlsPolicy>,
}

/// SES ReceiptRuleSetMetadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReceiptRuleSetMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// SES ReputationOptions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReputationOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_fresh_start: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reputation_metrics_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sending_enabled: Option<bool>,
}

/// SES S3Action.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct S3Action {
    pub bucket_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iam_role_arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_key_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_arn: Option<String>,
}

/// SES SNSAction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SNSAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<SNSActionEncoding>,
    pub topic_arn: String,
}

/// SES SNSDestination.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SNSDestination {
    #[serde(rename = "TopicARN")]
    pub topic_arn: String,
}

/// SES SendDataPoint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendDataPoint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounces: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complaints: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_attempts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejects: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// SES StopAction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StopAction {
    pub scope: StopScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_arn: Option<String>,
}

/// SES Template.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Template {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_part: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_part: Option<String>,
    pub template_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_part: Option<String>,
}

/// SES TemplateMetadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TemplateMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// SES TrackingOptions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TrackingOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_redirect_domain: Option<String>,
}

/// SES WorkmailAction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WorkmailAction {
    pub organization_arn: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_arn: Option<String>,
}
