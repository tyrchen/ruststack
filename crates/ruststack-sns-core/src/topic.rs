//! Topic record and attributes.

use std::collections::HashMap;

use crate::subscription::SubscriptionRecord;

/// A topic stored in the SNS state.
#[derive(Debug, Clone)]
pub struct TopicRecord {
    /// The topic ARN.
    pub arn: String,
    /// The topic name.
    pub name: String,
    /// Whether this is a FIFO topic.
    pub is_fifo: bool,
    /// Topic attributes.
    pub attributes: TopicAttributes,
    /// Subscriptions attached to this topic.
    pub subscriptions: Vec<SubscriptionRecord>,
    /// Resource tags.
    pub tags: HashMap<String, String>,
    /// Data protection policy JSON.
    pub data_protection_policy: Option<String>,
    /// Creation timestamp (epoch seconds).
    pub created_at: u64,
    /// Monotonically increasing counter for generating subscription ARNs.
    pub subscription_counter: u64,
}

/// Topic attributes following the AWS SNS attribute model.
#[derive(Debug, Clone)]
pub struct TopicAttributes {
    /// The display name for the topic.
    pub display_name: String,
    /// The topic access policy JSON.
    pub policy: Option<String>,
    /// The delivery policy JSON.
    pub delivery_policy: Option<String>,
    /// The effective delivery policy (computed).
    pub effective_delivery_policy: Option<String>,
    /// The KMS master key ID for encryption.
    pub kms_master_key_id: Option<String>,
    /// The signature version ("1" or "2").
    pub signature_version: String,
    /// Whether content-based deduplication is enabled (FIFO only).
    pub content_based_deduplication: bool,
    /// FIFO throughput limit.
    pub fifo_throughput_limit: Option<String>,
    /// The topic owner account ID.
    pub owner: String,
}

impl TopicAttributes {
    /// Build from the `CreateTopic` input attributes map.
    #[must_use]
    pub fn from_input(attrs: &HashMap<String, String>, is_fifo: bool, owner: &str) -> Self {
        Self {
            display_name: attrs.get("DisplayName").cloned().unwrap_or_default(),
            policy: attrs.get("Policy").cloned(),
            delivery_policy: attrs.get("DeliveryPolicy").cloned(),
            effective_delivery_policy: attrs.get("DeliveryPolicy").cloned(),
            kms_master_key_id: attrs.get("KmsMasterKeyId").cloned(),
            signature_version: attrs
                .get("SignatureVersion")
                .cloned()
                .unwrap_or_else(|| "1".to_owned()),
            content_based_deduplication: is_fifo
                && attrs
                    .get("ContentBasedDeduplication")
                    .is_some_and(|v| v.eq_ignore_ascii_case("true")),
            fifo_throughput_limit: attrs.get("FifoThroughputLimit").cloned(),
            owner: owner.to_owned(),
        }
    }

    /// Convert to the `HashMap` format returned by `GetTopicAttributes`.
    ///
    /// Includes computed attributes like `TopicArn`, `SubscriptionsConfirmed`, etc.
    #[must_use]
    pub fn to_map(&self, topic: &TopicRecord) -> HashMap<String, String> {
        let confirmed = topic.subscriptions.iter().filter(|s| s.confirmed).count();
        let pending = topic.subscriptions.iter().filter(|s| !s.confirmed).count();

        let mut map = HashMap::with_capacity(16);
        map.insert("TopicArn".to_owned(), topic.arn.clone());
        map.insert("Owner".to_owned(), self.owner.clone());
        map.insert("DisplayName".to_owned(), self.display_name.clone());
        map.insert("SubscriptionsConfirmed".to_owned(), confirmed.to_string());
        map.insert("SubscriptionsPending".to_owned(), pending.to_string());
        map.insert("SubscriptionsDeleted".to_owned(), "0".to_owned());

        if let Some(ref policy) = self.policy {
            map.insert("Policy".to_owned(), policy.clone());
        }
        if let Some(ref dp) = self.delivery_policy {
            map.insert("DeliveryPolicy".to_owned(), dp.clone());
        }
        if let Some(ref edp) = self.effective_delivery_policy {
            map.insert("EffectiveDeliveryPolicy".to_owned(), edp.clone());
        }
        if let Some(ref kms) = self.kms_master_key_id {
            map.insert("KmsMasterKeyId".to_owned(), kms.clone());
        }

        map.insert(
            "SignatureVersion".to_owned(),
            self.signature_version.clone(),
        );

        if topic.is_fifo {
            map.insert("FifoTopic".to_owned(), "true".to_owned());
            map.insert(
                "ContentBasedDeduplication".to_owned(),
                self.content_based_deduplication.to_string(),
            );
            if let Some(ref limit) = self.fifo_throughput_limit {
                map.insert("FifoThroughputLimit".to_owned(), limit.clone());
            }
        }

        map
    }
}
