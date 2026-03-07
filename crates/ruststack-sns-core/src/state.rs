//! Global SNS state management with `DashMap`.
//!
//! Provides concurrent access to topics and a reverse index
//! for looking up which topic a subscription belongs to.

use dashmap::DashMap;

use crate::topic::TopicRecord;

/// Thread-safe topic store using `DashMap` for concurrent access.
#[derive(Debug)]
pub struct TopicStore {
    /// topic_arn -> TopicRecord
    topics: DashMap<String, TopicRecord>,
    /// subscription_arn -> topic_arn (reverse index)
    subscription_index: DashMap<String, String>,
}

impl TopicStore {
    /// Create a new empty topic store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            topics: DashMap::new(),
            subscription_index: DashMap::new(),
        }
    }

    /// Get a read-only reference to a topic.
    #[must_use]
    pub fn get_topic(
        &self,
        arn: &str,
    ) -> Option<dashmap::mapref::one::Ref<'_, String, TopicRecord>> {
        self.topics.get(arn)
    }

    /// Get a mutable reference to a topic.
    #[must_use]
    pub fn get_topic_mut(
        &self,
        arn: &str,
    ) -> Option<dashmap::mapref::one::RefMut<'_, String, TopicRecord>> {
        self.topics.get_mut(arn)
    }

    /// Insert or replace a topic.
    pub fn insert_topic(&self, topic: TopicRecord) {
        self.topics.insert(topic.arn.clone(), topic);
    }

    /// Remove a topic and return it if it existed.
    ///
    /// Also cleans up the subscription index for all subscriptions
    /// that belonged to this topic.
    #[must_use]
    pub fn remove_topic(&self, arn: &str) -> Option<TopicRecord> {
        if let Some((_, topic)) = self.topics.remove(arn) {
            for sub in &topic.subscriptions {
                self.subscription_index.remove(&sub.arn);
            }
            Some(topic)
        } else {
            None
        }
    }

    /// List all topic ARNs.
    #[must_use]
    pub fn list_topics(&self) -> Vec<String> {
        self.topics.iter().map(|r| r.key().clone()).collect()
    }

    /// Find the topic ARN for a given subscription ARN.
    #[must_use]
    pub fn find_topic_for_subscription(&self, sub_arn: &str) -> Option<String> {
        self.subscription_index.get(sub_arn).map(|r| r.clone())
    }

    /// Register a subscription in the reverse index.
    pub fn add_subscription_index(&self, sub_arn: &str, topic_arn: &str) {
        self.subscription_index
            .insert(sub_arn.to_owned(), topic_arn.to_owned());
    }

    /// Remove a subscription from the reverse index.
    pub fn remove_subscription_index(&self, sub_arn: &str) {
        self.subscription_index.remove(sub_arn);
    }
}

impl Default for TopicStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::topic::TopicAttributes;

    fn make_topic(arn: &str, name: &str) -> TopicRecord {
        TopicRecord {
            arn: arn.to_owned(),
            name: name.to_owned(),
            is_fifo: false,
            attributes: TopicAttributes::from_input(&HashMap::new(), false, "000000000000"),
            subscriptions: Vec::new(),
            tags: HashMap::new(),
            data_protection_policy: None,
            created_at: 0,
            subscription_counter: 0,
            fifo_sequence_counter: std::sync::atomic::AtomicU64::new(0),
            fifo_dedup_cache: HashMap::new(),
        }
    }

    #[test]
    fn test_should_insert_and_get_topic() {
        let store = TopicStore::new();
        let topic = make_topic("arn:aws:sns:us-east-1:000000000000:test", "test");
        store.insert_topic(topic);
        assert!(
            store
                .get_topic("arn:aws:sns:us-east-1:000000000000:test")
                .is_some()
        );
    }

    #[test]
    fn test_should_remove_topic() {
        let store = TopicStore::new();
        let topic = make_topic("arn:aws:sns:us-east-1:000000000000:test", "test");
        store.insert_topic(topic);
        let removed = store.remove_topic("arn:aws:sns:us-east-1:000000000000:test");
        assert!(removed.is_some());
        assert!(
            store
                .get_topic("arn:aws:sns:us-east-1:000000000000:test")
                .is_none()
        );
    }

    #[test]
    fn test_should_list_topics() {
        let store = TopicStore::new();
        store.insert_topic(make_topic("arn:1", "t1"));
        store.insert_topic(make_topic("arn:2", "t2"));
        let topics = store.list_topics();
        assert_eq!(topics.len(), 2);
    }

    #[test]
    fn test_should_track_subscription_index() {
        let store = TopicStore::new();
        store.add_subscription_index("sub:1", "topic:1");
        assert_eq!(
            store.find_topic_for_subscription("sub:1"),
            Some("topic:1".to_owned())
        );
        store.remove_subscription_index("sub:1");
        assert!(store.find_topic_for_subscription("sub:1").is_none());
    }
}
