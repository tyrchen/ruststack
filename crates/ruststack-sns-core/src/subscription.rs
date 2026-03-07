//! Subscription record and attributes.

use std::collections::HashMap;
use std::fmt;

use ruststack_sns_model::error::SnsError;

/// A subscription stored in a topic.
#[derive(Debug, Clone)]
pub struct SubscriptionRecord {
    /// The subscription ARN.
    pub arn: String,
    /// The topic ARN this subscription belongs to.
    pub topic_arn: String,
    /// The subscription protocol.
    pub protocol: SubscriptionProtocol,
    /// The subscription endpoint (queue ARN, URL, email, etc.).
    pub endpoint: String,
    /// The subscription owner account ID.
    pub owner: String,
    /// Whether the subscription has been confirmed.
    pub confirmed: bool,
    /// Confirmation token for HTTP/HTTPS subscriptions.
    pub confirmation_token: Option<String>,
    /// Subscription attributes.
    pub attributes: SubscriptionAttributes,
}

/// Supported subscription protocols.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionProtocol {
    /// Amazon SQS queue.
    Sqs,
    /// HTTP endpoint.
    Http,
    /// HTTPS endpoint.
    Https,
    /// Email (human-readable).
    Email,
    /// Email (JSON-encoded).
    EmailJson,
    /// SMS text message.
    Sms,
    /// Mobile push (platform application endpoint).
    Application,
    /// AWS Lambda function.
    Lambda,
    /// Amazon Kinesis Data Firehose.
    Firehose,
}

impl SubscriptionProtocol {
    /// Parse a protocol string into a `SubscriptionProtocol`.
    pub fn parse(s: &str) -> Result<Self, SnsError> {
        match s {
            "sqs" => Ok(Self::Sqs),
            "http" => Ok(Self::Http),
            "https" => Ok(Self::Https),
            "email" => Ok(Self::Email),
            "email-json" => Ok(Self::EmailJson),
            "sms" => Ok(Self::Sms),
            "application" => Ok(Self::Application),
            "lambda" => Ok(Self::Lambda),
            "firehose" => Ok(Self::Firehose),
            _ => Err(SnsError::invalid_parameter(format!(
                "Invalid parameter: Protocol Reason: {s} is not a valid protocol"
            ))),
        }
    }

    /// Return the protocol as a string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sqs => "sqs",
            Self::Http => "http",
            Self::Https => "https",
            Self::Email => "email",
            Self::EmailJson => "email-json",
            Self::Sms => "sms",
            Self::Application => "application",
            Self::Lambda => "lambda",
            Self::Firehose => "firehose",
        }
    }

    /// Whether this protocol is auto-confirmed (no confirmation step needed).
    #[must_use]
    pub fn is_auto_confirmed(&self) -> bool {
        matches!(
            self,
            Self::Sqs
                | Self::Lambda
                | Self::Firehose
                | Self::Email
                | Self::EmailJson
                | Self::Sms
                | Self::Application
        )
    }
}

impl fmt::Display for SubscriptionProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Subscription-level attributes.
#[derive(Debug, Clone, Default)]
pub struct SubscriptionAttributes {
    /// Filter policy JSON.
    pub filter_policy: Option<String>,
    /// Filter policy scope.
    pub filter_policy_scope: FilterPolicyScope,
    /// Whether to deliver raw message (skip SNS envelope wrapping).
    pub raw_message_delivery: bool,
    /// Redrive policy JSON (dead-letter queue config).
    pub redrive_policy: Option<String>,
    /// Delivery policy JSON.
    pub delivery_policy: Option<String>,
    /// Subscription role ARN (for Firehose protocol).
    pub subscription_role_arn: Option<String>,
}

/// Scope for filter policy evaluation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum FilterPolicyScope {
    /// Filter on message attributes (default).
    #[default]
    MessageAttributes,
    /// Filter on message body.
    MessageBody,
}

impl FilterPolicyScope {
    /// Parse a scope string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("MessageBody") {
            Self::MessageBody
        } else {
            Self::MessageAttributes
        }
    }

    /// Return the scope as a string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MessageAttributes => "MessageAttributes",
            Self::MessageBody => "MessageBody",
        }
    }
}

impl SubscriptionAttributes {
    /// Build subscription attributes from form parameters.
    pub fn from_input(attrs: &HashMap<String, String>) -> Result<Self, SnsError> {
        let filter_policy = attrs.get("FilterPolicy").cloned();
        let filter_policy_scope = attrs
            .get("FilterPolicyScope")
            .map(|s| FilterPolicyScope::parse(s))
            .unwrap_or_default();
        let raw_message_delivery = attrs
            .get("RawMessageDelivery")
            .is_some_and(|v| v.eq_ignore_ascii_case("true"));
        let redrive_policy = attrs.get("RedrivePolicy").cloned();
        let delivery_policy = attrs.get("DeliveryPolicy").cloned();
        let subscription_role_arn = attrs.get("SubscriptionRoleArn").cloned();

        // Validate filter policy is valid JSON if present.
        if let Some(ref fp) = filter_policy {
            if serde_json::from_str::<serde_json::Value>(fp).is_err() {
                return Err(SnsError::invalid_parameter(
                    "Invalid parameter: FilterPolicy Reason: failed to parse JSON",
                ));
            }
        }

        Ok(Self {
            filter_policy,
            filter_policy_scope,
            raw_message_delivery,
            redrive_policy,
            delivery_policy,
            subscription_role_arn,
        })
    }

    /// Convert to the `HashMap` format returned by `GetSubscriptionAttributes`.
    #[must_use]
    pub fn to_map(&self, sub: &SubscriptionRecord) -> HashMap<String, String> {
        let mut map = HashMap::with_capacity(16);
        map.insert("SubscriptionArn".to_owned(), sub.arn.clone());
        map.insert("TopicArn".to_owned(), sub.topic_arn.clone());
        map.insert("Protocol".to_owned(), sub.protocol.as_str().to_owned());
        map.insert("Endpoint".to_owned(), sub.endpoint.clone());
        map.insert("Owner".to_owned(), sub.owner.clone());
        map.insert("ConfirmationWasAuthenticated".to_owned(), "true".to_owned());
        map.insert(
            "PendingConfirmation".to_owned(),
            (!sub.confirmed).to_string(),
        );
        map.insert(
            "RawMessageDelivery".to_owned(),
            self.raw_message_delivery.to_string(),
        );

        if let Some(ref fp) = self.filter_policy {
            map.insert("FilterPolicy".to_owned(), fp.clone());
        }

        map.insert(
            "FilterPolicyScope".to_owned(),
            self.filter_policy_scope.as_str().to_owned(),
        );

        if let Some(ref rp) = self.redrive_policy {
            map.insert("RedrivePolicy".to_owned(), rp.clone());
        }
        if let Some(ref dp) = self.delivery_policy {
            map.insert("DeliveryPolicy".to_owned(), dp.clone());
        }
        if let Some(ref role) = self.subscription_role_arn {
            map.insert("SubscriptionRoleArn".to_owned(), role.clone());
        }

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_parse_valid_protocols() {
        assert_eq!(
            SubscriptionProtocol::parse("sqs").unwrap(),
            SubscriptionProtocol::Sqs
        );
        assert_eq!(
            SubscriptionProtocol::parse("http").unwrap(),
            SubscriptionProtocol::Http
        );
        assert_eq!(
            SubscriptionProtocol::parse("lambda").unwrap(),
            SubscriptionProtocol::Lambda
        );
    }

    #[test]
    fn test_should_reject_invalid_protocol() {
        let err = SubscriptionProtocol::parse("invalid").unwrap_err();
        assert!(err.message.contains("invalid"));
    }

    #[test]
    fn test_should_auto_confirm_sqs() {
        assert!(SubscriptionProtocol::Sqs.is_auto_confirmed());
        assert!(!SubscriptionProtocol::Http.is_auto_confirmed());
    }

    #[test]
    fn test_should_parse_filter_policy_scope() {
        assert_eq!(
            FilterPolicyScope::parse("MessageBody"),
            FilterPolicyScope::MessageBody
        );
        assert_eq!(
            FilterPolicyScope::parse("MessageAttributes"),
            FilterPolicyScope::MessageAttributes
        );
        assert_eq!(
            FilterPolicyScope::parse("unknown"),
            FilterPolicyScope::MessageAttributes
        );
    }

    #[test]
    fn test_should_build_attributes_from_input() {
        let mut attrs = HashMap::new();
        attrs.insert("RawMessageDelivery".to_owned(), "true".to_owned());
        attrs.insert(
            "FilterPolicy".to_owned(),
            r#"{"key": ["value"]}"#.to_owned(),
        );
        let sub_attrs = SubscriptionAttributes::from_input(&attrs).unwrap();
        assert!(sub_attrs.raw_message_delivery);
        assert!(sub_attrs.filter_policy.is_some());
    }

    #[test]
    fn test_should_reject_invalid_filter_policy_json() {
        let mut attrs = HashMap::new();
        attrs.insert("FilterPolicy".to_owned(), "not-json".to_owned());
        let err = SubscriptionAttributes::from_input(&attrs).unwrap_err();
        assert!(err.message.contains("FilterPolicy"));
    }
}
