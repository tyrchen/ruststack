//! SNS integration tests against a running `Rustack` server.
//!
//! These tests cover topic lifecycle, subscriptions, publishing,
//! fan-out to SQS, tags, batch publishing, and FIFO topics.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use aws_sdk_sns as sns;

    use crate::{create_test_queue, delete_test_queue, sns_client, sqs_client};

    /// Generate a unique topic name for an SNS test.
    fn test_topic_name(prefix: &str) -> String {
        let id = &uuid::Uuid::new_v4().to_string()[..8];
        format!("test-{prefix}-{id}")
    }

    /// Create a topic and return its ARN. Caller is responsible for cleanup.
    async fn create_test_topic(client: &sns::Client, prefix: &str) -> String {
        let name = test_topic_name(prefix);
        client
            .create_topic()
            .name(&name)
            .send()
            .await
            .unwrap_or_else(|e| panic!("failed to create topic {name}: {e}"))
            .topic_arn()
            .unwrap()
            .to_string()
    }

    /// Delete a topic by ARN.
    async fn delete_test_topic(client: &sns::Client, topic_arn: &str) {
        let _ = client.delete_topic().topic_arn(topic_arn).send().await;
    }

    // ---------------------------------------------------------------------------
    // Topic CRUD
    // ---------------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_create_and_delete_topic() {
        let client = sns_client();
        let arn = create_test_topic(&client, "lifecycle").await;

        // Verify ARN looks reasonable.
        assert!(arn.contains(":sns:"), "ARN should contain :sns:");
        assert!(arn.starts_with("arn:aws:sns:"));

        delete_test_topic(&client, &arn).await;

        // Verify topic is gone from the list.
        let topics = client.list_topics().send().await.unwrap();
        let arns: Vec<_> = topics
            .topics()
            .iter()
            .filter_map(|t| t.topic_arn())
            .collect();
        assert!(!arns.contains(&&*arn));
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_list_topics() {
        let client = sns_client();
        let first_arn = create_test_topic(&client, "list-a").await;
        let second_arn = create_test_topic(&client, "list-b").await;

        let topics = client.list_topics().send().await.unwrap();
        let topic_arns: Vec<_> = topics
            .topics()
            .iter()
            .filter_map(|t| t.topic_arn())
            .collect();

        assert!(
            topic_arns.contains(&&*first_arn),
            "missing topic {first_arn}"
        );
        assert!(
            topic_arns.contains(&&*second_arn),
            "missing topic {second_arn}"
        );

        delete_test_topic(&client, &first_arn).await;
        delete_test_topic(&client, &second_arn).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_get_and_set_topic_attributes() {
        let client = sns_client();
        let arn = create_test_topic(&client, "attrs").await;

        // Get initial attributes.
        let attrs = client
            .get_topic_attributes()
            .topic_arn(&arn)
            .send()
            .await
            .unwrap();

        let map = attrs.attributes().unwrap();
        assert!(map.contains_key("TopicArn"), "missing TopicArn attribute");

        // Set DisplayName.
        client
            .set_topic_attributes()
            .topic_arn(&arn)
            .attribute_name("DisplayName")
            .attribute_value("My Test Topic")
            .send()
            .await
            .unwrap();

        // Verify update.
        let attrs2 = client
            .get_topic_attributes()
            .topic_arn(&arn)
            .send()
            .await
            .unwrap();

        let map2 = attrs2.attributes().unwrap();
        assert_eq!(map2.get("DisplayName").unwrap(), "My Test Topic");

        delete_test_topic(&client, &arn).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_create_topic_idempotent() {
        let client = sns_client();
        let name = test_topic_name("idempotent");

        let arn1 = client
            .create_topic()
            .name(&name)
            .send()
            .await
            .unwrap()
            .topic_arn()
            .unwrap()
            .to_string();

        let arn2 = client
            .create_topic()
            .name(&name)
            .send()
            .await
            .unwrap()
            .topic_arn()
            .unwrap()
            .to_string();

        assert_eq!(arn1, arn2);

        delete_test_topic(&client, &arn1).await;
    }

    // ---------------------------------------------------------------------------
    // Subscriptions
    // ---------------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_subscribe_and_unsubscribe() {
        let client = sns_client();
        let sqs = sqs_client();

        let arn = create_test_topic(&client, "sub").await;
        let (_, queue_url) = create_test_queue(&sqs, "sns-sub").await;

        // Get the queue ARN.
        let queue_attrs = sqs
            .get_queue_attributes()
            .queue_url(&queue_url)
            .attribute_names(aws_sdk_sqs::types::QueueAttributeName::QueueArn)
            .send()
            .await
            .unwrap();
        let queue_arn = queue_attrs
            .attributes()
            .unwrap()
            .get(&aws_sdk_sqs::types::QueueAttributeName::QueueArn)
            .unwrap()
            .clone();

        // Subscribe queue to topic.
        let sub = client
            .subscribe()
            .topic_arn(&arn)
            .protocol("sqs")
            .endpoint(&queue_arn)
            .send()
            .await
            .unwrap();

        let sub_arn = sub.subscription_arn().unwrap().to_string();
        assert!(
            sub_arn.contains(":sns:"),
            "subscription ARN should contain :sns:"
        );

        // Verify subscription appears in list.
        let subs = client.list_subscriptions().send().await.unwrap();
        let sub_arns: Vec<_> = subs
            .subscriptions()
            .iter()
            .filter_map(|s| s.subscription_arn())
            .collect();
        assert!(
            sub_arns.contains(&&*sub_arn),
            "subscription {sub_arn} should be in list"
        );

        // Unsubscribe.
        client
            .unsubscribe()
            .subscription_arn(&sub_arn)
            .send()
            .await
            .unwrap();

        // Verify subscription is gone.
        let subs2 = client.list_subscriptions().send().await.unwrap();
        let sub_arns2: Vec<_> = subs2
            .subscriptions()
            .iter()
            .filter_map(|s| s.subscription_arn())
            .collect();
        assert!(
            !sub_arns2.contains(&&*sub_arn),
            "subscription {sub_arn} should be removed"
        );

        delete_test_topic(&client, &arn).await;
        delete_test_queue(&sqs, &queue_url).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_list_subscriptions_by_topic() {
        let client = sns_client();
        let sqs = sqs_client();

        let arn = create_test_topic(&client, "sub-by-topic").await;
        let (_, queue_url) = create_test_queue(&sqs, "sns-sub-by-topic").await;

        // Get queue ARN.
        let queue_attrs = sqs
            .get_queue_attributes()
            .queue_url(&queue_url)
            .attribute_names(aws_sdk_sqs::types::QueueAttributeName::QueueArn)
            .send()
            .await
            .unwrap();
        let queue_arn = queue_attrs
            .attributes()
            .unwrap()
            .get(&aws_sdk_sqs::types::QueueAttributeName::QueueArn)
            .unwrap()
            .clone();

        // Subscribe.
        let sub = client
            .subscribe()
            .topic_arn(&arn)
            .protocol("sqs")
            .endpoint(&queue_arn)
            .send()
            .await
            .unwrap();
        let sub_arn = sub.subscription_arn().unwrap().to_string();

        // List subscriptions by topic.
        let subs = client
            .list_subscriptions_by_topic()
            .topic_arn(&arn)
            .send()
            .await
            .unwrap();

        let sub_arns: Vec<_> = subs
            .subscriptions()
            .iter()
            .filter_map(|s| s.subscription_arn())
            .collect();
        assert!(
            sub_arns.contains(&&*sub_arn),
            "subscription should be listed under topic"
        );

        // Cleanup.
        client
            .unsubscribe()
            .subscription_arn(&sub_arn)
            .send()
            .await
            .unwrap();
        delete_test_topic(&client, &arn).await;
        delete_test_queue(&sqs, &queue_url).await;
    }

    // ---------------------------------------------------------------------------
    // Publish & Fan-out
    // ---------------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_publish_to_topic() {
        let client = sns_client();
        let arn = create_test_topic(&client, "publish").await;

        let result = client
            .publish()
            .topic_arn(&arn)
            .message("hello from SNS")
            .send()
            .await
            .unwrap();

        assert!(
            result.message_id().is_some(),
            "publish should return a message ID"
        );

        delete_test_topic(&client, &arn).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_fan_out_to_sqs() {
        let client = sns_client();
        let sqs = sqs_client();

        let topic_arn = create_test_topic(&client, "fanout").await;
        let (_, queue_url) = create_test_queue(&sqs, "sns-fanout").await;

        // Get queue ARN.
        let queue_attrs = sqs
            .get_queue_attributes()
            .queue_url(&queue_url)
            .attribute_names(aws_sdk_sqs::types::QueueAttributeName::QueueArn)
            .send()
            .await
            .unwrap();
        let queue_arn = queue_attrs
            .attributes()
            .unwrap()
            .get(&aws_sdk_sqs::types::QueueAttributeName::QueueArn)
            .unwrap()
            .clone();

        // Subscribe queue to topic.
        let sub = client
            .subscribe()
            .topic_arn(&topic_arn)
            .protocol("sqs")
            .endpoint(&queue_arn)
            .send()
            .await
            .unwrap();
        let sub_arn = sub.subscription_arn().unwrap().to_string();

        // Publish a message.
        let published_message = "hello fan-out";
        client
            .publish()
            .topic_arn(&topic_arn)
            .message(published_message)
            .send()
            .await
            .unwrap();

        // Give the fan-out a moment to deliver.
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Receive from SQS queue.
        let recv = sqs
            .receive_message()
            .queue_url(&queue_url)
            .max_number_of_messages(1)
            .wait_time_seconds(5)
            .send()
            .await
            .unwrap();

        let messages = recv.messages();
        assert_eq!(messages.len(), 1, "expected 1 message from fan-out");

        // The SQS message body is the SNS envelope JSON.
        let body = messages[0].body().unwrap();
        let envelope: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(envelope["Type"], "Notification");
        assert_eq!(envelope["Message"], published_message);
        assert!(
            envelope["TopicArn"].as_str().unwrap().contains("fanout"),
            "envelope TopicArn should match"
        );

        // Cleanup.
        client
            .unsubscribe()
            .subscription_arn(&sub_arn)
            .send()
            .await
            .unwrap();
        delete_test_topic(&client, &topic_arn).await;
        delete_test_queue(&sqs, &queue_url).await;
    }

    // ---------------------------------------------------------------------------
    // Tags
    // ---------------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_tag_and_untag_topic() {
        let client = sns_client();
        let arn = create_test_topic(&client, "tags").await;

        // Add tags.
        let env_tag = sns::types::Tag::builder()
            .key("env")
            .value("test")
            .build()
            .unwrap();
        let project_tag = sns::types::Tag::builder()
            .key("project")
            .value("rustack")
            .build()
            .unwrap();

        client
            .tag_resource()
            .resource_arn(&arn)
            .tags(env_tag)
            .tags(project_tag)
            .send()
            .await
            .unwrap();

        // List tags.
        let result = client
            .list_tags_for_resource()
            .resource_arn(&arn)
            .send()
            .await
            .unwrap();

        let tag_map: HashMap<_, _> = result
            .tags()
            .iter()
            .map(|t| (t.key().to_string(), t.value().to_string()))
            .collect();
        assert_eq!(tag_map.get("env").unwrap(), "test");
        assert_eq!(tag_map.get("project").unwrap(), "rustack");

        // Untag.
        client
            .untag_resource()
            .resource_arn(&arn)
            .tag_keys("env")
            .send()
            .await
            .unwrap();

        // Verify removal.
        let result2 = client
            .list_tags_for_resource()
            .resource_arn(&arn)
            .send()
            .await
            .unwrap();

        let remaining_tags: HashMap<_, _> = result2
            .tags()
            .iter()
            .map(|t| (t.key().to_string(), t.value().to_string()))
            .collect();
        assert!(
            !remaining_tags.contains_key("env"),
            "env tag should be removed"
        );
        assert!(
            remaining_tags.contains_key("project"),
            "project tag should still exist"
        );

        delete_test_topic(&client, &arn).await;
    }

    // ---------------------------------------------------------------------------
    // Publish Batch
    // ---------------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_publish_batch() {
        let client = sns_client();
        let arn = create_test_topic(&client, "batch").await;

        let entries: Vec<_> = (0..3)
            .map(|i| {
                sns::types::PublishBatchRequestEntry::builder()
                    .id(format!("msg-{i}"))
                    .message(format!("batch message {i}"))
                    .build()
                    .unwrap()
            })
            .collect();

        let result = client
            .publish_batch()
            .topic_arn(&arn)
            .set_publish_batch_request_entries(Some(entries))
            .send()
            .await
            .unwrap();

        assert_eq!(
            result.successful().len(),
            3,
            "all 3 messages should succeed"
        );
        for entry in result.successful() {
            assert!(
                entry.message_id().is_some(),
                "each batch entry should have a message ID"
            );
        }
        assert!(result.failed().is_empty(), "no messages should have failed");

        delete_test_topic(&client, &arn).await;
    }

    // ---------------------------------------------------------------------------
    // FIFO Topics
    // ---------------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_create_fifo_topic() {
        let client = sns_client();
        let id = &uuid::Uuid::new_v4().to_string()[..8];
        let name = format!("test-fifo-{id}.fifo");

        let arn = client
            .create_topic()
            .name(&name)
            .attributes("FifoTopic", "true")
            .send()
            .await
            .unwrap()
            .topic_arn()
            .unwrap()
            .to_string();

        assert!(
            arn.as_bytes().ends_with(b".fifo"),
            "FIFO topic ARN should end with .fifo"
        );

        // Verify attribute.
        let attrs = client
            .get_topic_attributes()
            .topic_arn(&arn)
            .send()
            .await
            .unwrap();

        let map = attrs.attributes().unwrap();
        assert_eq!(
            map.get("FifoTopic").unwrap(),
            "true",
            "FifoTopic attribute should be true"
        );

        delete_test_topic(&client, &arn).await;
    }
}
