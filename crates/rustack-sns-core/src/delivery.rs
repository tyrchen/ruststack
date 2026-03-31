//! SNS message delivery and envelope formatting.
//!
//! Builds the SNS message envelope JSON that wraps published messages
//! before delivering them to SQS queues and HTTP endpoints.

use std::collections::HashMap;

use rustack_sns_model::types::MessageAttributeValue;
use serde::Serialize;

/// SNS message envelope sent to SQS queues and HTTP endpoints.
///
/// This follows the AWS SNS notification JSON format with specific
/// casing for `SigningCertURL` and `UnsubscribeURL` (capital "URL").
#[derive(Debug, Clone, Serialize)]
pub struct SnsMessageEnvelope {
    /// The message type (always "Notification").
    #[serde(rename = "Type")]
    pub message_type: String,
    /// The unique message ID.
    #[serde(rename = "MessageId")]
    pub message_id: String,
    /// The topic ARN that published the message.
    #[serde(rename = "TopicArn")]
    pub topic_arn: String,
    /// The message subject (optional).
    #[serde(rename = "Subject", skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The message body.
    #[serde(rename = "Message")]
    pub message: String,
    /// The timestamp in ISO 8601 format.
    #[serde(rename = "Timestamp")]
    pub timestamp: String,
    /// The signature version.
    #[serde(rename = "SignatureVersion")]
    pub signature_version: String,
    /// The message signature (stub in local mode).
    #[serde(rename = "Signature")]
    pub signature: String,
    /// URL for the signing certificate.
    #[serde(rename = "SigningCertURL")]
    pub signing_cert_url: String,
    /// URL to unsubscribe.
    #[serde(rename = "UnsubscribeURL")]
    pub unsubscribe_url: String,
    /// Message attributes (if any).
    #[serde(rename = "MessageAttributes", skip_serializing_if = "Option::is_none")]
    pub message_attributes: Option<HashMap<String, SnsMessageAttributeEnvelope>>,
}

/// A single message attribute in the SNS envelope format.
#[derive(Debug, Clone, Serialize)]
pub struct SnsMessageAttributeEnvelope {
    /// The attribute data type.
    #[serde(rename = "Type")]
    pub data_type: String,
    /// The attribute value.
    #[serde(rename = "Value")]
    pub value: String,
}

/// Parameters for building an SNS envelope.
#[derive(Debug)]
pub struct EnvelopeParams<'a> {
    /// The unique message ID.
    pub message_id: &'a str,
    /// The topic ARN.
    pub topic_arn: &'a str,
    /// Optional message subject.
    pub subject: Option<&'a str>,
    /// The message body.
    pub message: &'a str,
    /// Message attributes.
    pub message_attributes: &'a HashMap<String, MessageAttributeValue>,
    /// The AWS region.
    pub region: &'a str,
    /// The service host.
    pub host: &'a str,
    /// The service port.
    pub port: u16,
}

/// Build an SNS message envelope JSON string.
///
/// This creates the standard SNS notification envelope that wraps
/// the published message. Used when delivering to SQS (non-raw) and
/// HTTP/HTTPS endpoints.
pub fn build_sns_envelope(params: &EnvelopeParams<'_>) -> Result<String, serde_json::Error> {
    let attrs = if params.message_attributes.is_empty() {
        None
    } else {
        Some(
            params
                .message_attributes
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        SnsMessageAttributeEnvelope {
                            data_type: v.data_type.clone(),
                            value: v.string_value.clone().unwrap_or_default(),
                        },
                    )
                })
                .collect(),
        )
    };

    let envelope = SnsMessageEnvelope {
        message_type: "Notification".to_owned(),
        message_id: params.message_id.to_owned(),
        topic_arn: params.topic_arn.to_owned(),
        subject: params.subject.map(String::from),
        message: params.message.to_owned(),
        timestamp: chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string(),
        signature_version: "1".to_owned(),
        signature: "EXAMPLE".to_owned(),
        signing_cert_url: format!(
            "https://sns.{}.amazonaws.com/\
             SimpleNotificationService-0000000000000000000000000000000000.pem",
            params.region
        ),
        unsubscribe_url: format!(
            "http://{}:{}/?Action=Unsubscribe&SubscriptionArn={}",
            params.host, params.port, params.topic_arn
        ),
        message_attributes: attrs,
    };

    serde_json::to_string(&envelope)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_build_envelope_without_attributes() {
        let params = EnvelopeParams {
            message_id: "msg-123",
            topic_arn: "arn:aws:sns:us-east-1:000000000000:test",
            subject: Some("Test Subject"),
            message: "Hello World",
            message_attributes: &HashMap::new(),
            region: "us-east-1",
            host: "localhost",
            port: 4566,
        };
        let json = build_sns_envelope(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["Type"], "Notification");
        assert_eq!(parsed["MessageId"], "msg-123");
        assert_eq!(parsed["Message"], "Hello World");
        assert_eq!(parsed["Subject"], "Test Subject");
        assert!(parsed.get("MessageAttributes").is_none());
    }

    #[test]
    fn test_should_build_envelope_with_attributes() {
        let mut attrs = HashMap::new();
        attrs.insert(
            "key1".to_owned(),
            MessageAttributeValue {
                data_type: "String".to_owned(),
                string_value: Some("val1".to_owned()),
                binary_value: None,
            },
        );
        let params = EnvelopeParams {
            message_id: "msg-456",
            topic_arn: "arn:aws:sns:us-east-1:000000000000:test",
            subject: None,
            message: "Message body",
            message_attributes: &attrs,
            region: "us-east-1",
            host: "localhost",
            port: 4566,
        };
        let json = build_sns_envelope(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["MessageAttributes"]["key1"]["Type"], "String");
        assert_eq!(parsed["MessageAttributes"]["key1"]["Value"], "val1");
    }

    #[test]
    fn test_should_not_include_subject_when_none() {
        let params = EnvelopeParams {
            message_id: "msg-789",
            topic_arn: "arn:aws:sns:us-east-1:000000000000:test",
            subject: None,
            message: "body",
            message_attributes: &HashMap::new(),
            region: "us-east-1",
            host: "localhost",
            port: 4566,
        };
        let json = build_sns_envelope(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("Subject").is_none());
    }
}
