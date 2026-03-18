//! Integration tests for EventBridge service.
//!
//! These tests require a running RustStack server at `localhost:4566`.
//! Run with: `cargo test -p ruststack-integration -- events --ignored`

#[cfg(test)]
mod tests {
    use aws_sdk_eventbridge as eventbridge;
    use aws_sdk_eventbridge::types::{PutEventsRequestEntry, Target};

    use crate::{create_test_queue, delete_test_queue, events_client, sqs_client};

    // -----------------------------------------------------------------------
    // Event bus lifecycle
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_events_should_list_default_bus() {
        let client = events_client();
        let result = client.list_event_buses().send().await.unwrap();
        let buses = result.event_buses();
        assert!(
            buses.iter().any(|b| b.name() == Some("default")),
            "default bus should exist"
        );
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_events_should_create_and_delete_bus() {
        let client = events_client();
        let bus_name = format!(
            "test-bus-{}",
            uuid::Uuid::new_v4().to_string()[..8].to_owned()
        );

        // Create
        let create = client
            .create_event_bus()
            .name(&bus_name)
            .send()
            .await
            .unwrap();
        assert!(create.event_bus_arn().is_some());

        // Describe
        let describe = client
            .describe_event_bus()
            .name(&bus_name)
            .send()
            .await
            .unwrap();
        assert_eq!(describe.name(), Some(bus_name.as_str()));

        // List should include it
        let list = client.list_event_buses().send().await.unwrap();
        assert!(
            list.event_buses()
                .iter()
                .any(|b| b.name() == Some(bus_name.as_str())),
            "custom bus should appear in list"
        );

        // Delete
        client
            .delete_event_bus()
            .name(&bus_name)
            .send()
            .await
            .unwrap();

        // Should be gone
        let result = client.describe_event_bus().name(&bus_name).send().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_events_should_not_delete_default_bus() {
        let client = events_client();
        let result = client.delete_event_bus().name("default").send().await;
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Rule lifecycle
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_events_should_crud_rule() {
        let client = events_client();
        let rule_name = format!(
            "test-rule-{}",
            uuid::Uuid::new_v4().to_string()[..8].to_owned()
        );

        // PutRule
        let put = client
            .put_rule()
            .name(&rule_name)
            .event_pattern(r#"{"source":["my.app"]}"#)
            .description("test rule")
            .send()
            .await
            .unwrap();
        assert!(put.rule_arn().is_some());

        // DescribeRule
        let describe = client
            .describe_rule()
            .name(&rule_name)
            .send()
            .await
            .unwrap();
        assert_eq!(describe.name(), Some(rule_name.as_str()));
        assert_eq!(
            describe.state(),
            Some(&eventbridge::types::RuleState::Enabled)
        );
        assert!(describe.event_pattern().is_some());

        // ListRules
        let list = client.list_rules().send().await.unwrap();
        assert!(
            list.rules()
                .iter()
                .any(|r| r.name() == Some(rule_name.as_str())),
            "rule should appear in list"
        );

        // DisableRule
        client.disable_rule().name(&rule_name).send().await.unwrap();
        let describe = client
            .describe_rule()
            .name(&rule_name)
            .send()
            .await
            .unwrap();
        assert_eq!(
            describe.state(),
            Some(&eventbridge::types::RuleState::Disabled)
        );

        // EnableRule
        client.enable_rule().name(&rule_name).send().await.unwrap();

        // DeleteRule
        client.delete_rule().name(&rule_name).send().await.unwrap();
    }

    // -----------------------------------------------------------------------
    // Target lifecycle
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_events_should_crud_targets() {
        let client = events_client();
        let rule_name = format!(
            "test-rule-tgt-{}",
            uuid::Uuid::new_v4().to_string()[..8].to_owned()
        );

        // Create rule first
        client
            .put_rule()
            .name(&rule_name)
            .event_pattern(r#"{"source":["my.app"]}"#)
            .send()
            .await
            .unwrap();

        // PutTargets
        let target = Target::builder()
            .id("sqs-1")
            .arn("arn:aws:sqs:us-east-1:000000000000:test-queue")
            .build()
            .unwrap();
        let put = client
            .put_targets()
            .rule(&rule_name)
            .targets(target)
            .send()
            .await
            .unwrap();
        assert_eq!(put.failed_entry_count(), 0);

        // ListTargetsByRule
        let list = client
            .list_targets_by_rule()
            .rule(&rule_name)
            .send()
            .await
            .unwrap();
        assert_eq!(list.targets().len(), 1);
        assert_eq!(list.targets()[0].id(), "sqs-1");

        // RemoveTargets
        let remove = client
            .remove_targets()
            .rule(&rule_name)
            .ids("sqs-1")
            .send()
            .await
            .unwrap();
        assert_eq!(remove.failed_entry_count(), 0);

        // Cleanup
        client.delete_rule().name(&rule_name).send().await.unwrap();
    }

    // -----------------------------------------------------------------------
    // TestEventPattern
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_events_should_match_pattern() {
        let client = events_client();

        let result = client
            .test_event_pattern()
            .event(r#"{"source":"my.app","detail-type":"test","detail":{"status":"active"}}"#)
            .event_pattern(r#"{"detail":{"status":["active"]}}"#)
            .send()
            .await
            .unwrap();
        assert!(result.result());
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_events_should_not_match_pattern() {
        let client = events_client();

        let result = client
            .test_event_pattern()
            .event(r#"{"source":"my.app","detail-type":"test","detail":{"status":"inactive"}}"#)
            .event_pattern(r#"{"detail":{"status":["active"]}}"#)
            .send()
            .await
            .unwrap();
        assert!(!result.result());
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_events_should_match_prefix_pattern() {
        let client = events_client();

        let result = client
            .test_event_pattern()
            .event(r#"{"source":"my.app","detail":{"region":"us-west-2"}}"#)
            .event_pattern(r#"{"detail":{"region":[{"prefix":"us-"}]}}"#)
            .send()
            .await
            .unwrap();
        assert!(result.result());
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_events_should_match_numeric_pattern() {
        let client = events_client();

        let result = client
            .test_event_pattern()
            .event(r#"{"source":"my.app","detail":{"amount":150}}"#)
            .event_pattern(r#"{"detail":{"amount":[{"numeric":[">",100]}]}}"#)
            .send()
            .await
            .unwrap();
        assert!(result.result());
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_events_should_match_anything_but_pattern() {
        let client = events_client();

        let result = client
            .test_event_pattern()
            .event(r#"{"source":"my.app","detail":{"status":"active"}}"#)
            .event_pattern(r#"{"detail":{"status":[{"anything-but":"cancelled"}]}}"#)
            .send()
            .await
            .unwrap();
        assert!(result.result());
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_events_should_match_exists_pattern() {
        let client = events_client();

        let result = client
            .test_event_pattern()
            .event(r#"{"source":"my.app","detail":{"user_id":"abc"}}"#)
            .event_pattern(r#"{"detail":{"user_id":[{"exists":true}]}}"#)
            .send()
            .await
            .unwrap();
        assert!(result.result());
    }

    // -----------------------------------------------------------------------
    // PutEvents with SQS delivery
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_events_should_deliver_to_sqs_target() {
        let events = events_client();
        let sqs = sqs_client();

        // Create SQS queue
        let (queue_name, queue_url) = create_test_queue(&sqs, "events-tgt").await;

        let rule_name = format!(
            "test-delivery-{}",
            uuid::Uuid::new_v4().to_string()[..8].to_owned()
        );

        // Create rule
        events
            .put_rule()
            .name(&rule_name)
            .event_pattern(r#"{"source":["my.delivery.app"]}"#)
            .send()
            .await
            .unwrap();

        // Add SQS target
        let target_arn = format!("arn:aws:sqs:us-east-1:000000000000:{queue_name}");
        let target = Target::builder()
            .id("sqs-target")
            .arn(&target_arn)
            .build()
            .unwrap();
        events
            .put_targets()
            .rule(&rule_name)
            .targets(target)
            .send()
            .await
            .unwrap();

        // PutEvents
        let entry = PutEventsRequestEntry::builder()
            .source("my.delivery.app")
            .detail_type("TestDelivery")
            .detail(r#"{"key":"value"}"#)
            .build();
        let put_result = events.put_events().entries(entry).send().await.unwrap();
        assert_eq!(put_result.failed_entry_count(), 0);
        assert_eq!(put_result.entries().len(), 1);
        assert!(put_result.entries()[0].event_id().is_some());

        // Verify SQS received the event
        let recv = sqs
            .receive_message()
            .queue_url(&queue_url)
            .wait_time_seconds(5)
            .send()
            .await
            .unwrap();
        let messages = recv.messages();
        assert_eq!(messages.len(), 1, "SQS should have received 1 message");

        // Verify the message contains the event envelope
        let body = messages[0].body().unwrap();
        let envelope: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(envelope["source"], "my.delivery.app");
        assert_eq!(envelope["detail-type"], "TestDelivery");
        assert_eq!(envelope["detail"]["key"], "value");
        assert_eq!(envelope["version"], "0");

        // Cleanup
        events
            .remove_targets()
            .rule(&rule_name)
            .ids("sqs-target")
            .send()
            .await
            .unwrap();
        events.delete_rule().name(&rule_name).send().await.unwrap();
        delete_test_queue(&sqs, &queue_url).await;
    }

    // -----------------------------------------------------------------------
    // Tags
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_events_should_tag_and_untag_rule() {
        let client = events_client();
        let rule_name = format!(
            "test-tag-{}",
            uuid::Uuid::new_v4().to_string()[..8].to_owned()
        );

        // Create rule
        let put = client
            .put_rule()
            .name(&rule_name)
            .event_pattern(r#"{"source":["my.app"]}"#)
            .send()
            .await
            .unwrap();
        let rule_arn = put.rule_arn().unwrap().to_string();

        // Tag
        client
            .tag_resource()
            .resource_arn(&rule_arn)
            .tags(
                eventbridge::types::Tag::builder()
                    .key("env")
                    .value("test")
                    .build()
                    .unwrap(),
            )
            .send()
            .await
            .unwrap();

        // ListTags
        let tags = client
            .list_tags_for_resource()
            .resource_arn(&rule_arn)
            .send()
            .await
            .unwrap();
        assert!(tags.tags().iter().any(|t| t.key() == "env"));

        // Untag
        client
            .untag_resource()
            .resource_arn(&rule_arn)
            .tag_keys("env")
            .send()
            .await
            .unwrap();

        // Verify removed
        let tags = client
            .list_tags_for_resource()
            .resource_arn(&rule_arn)
            .send()
            .await
            .unwrap();
        assert!(!tags.tags().iter().any(|t| t.key() == "env"));

        // Cleanup
        client.delete_rule().name(&rule_name).send().await.unwrap();
    }

    // -----------------------------------------------------------------------
    // PutEvents - non-matching pattern should not deliver
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_events_should_not_deliver_non_matching() {
        let events = events_client();
        let sqs = sqs_client();

        let (queue_name, queue_url) = create_test_queue(&sqs, "events-nomatch").await;
        let rule_name = format!(
            "test-nomatch-{}",
            uuid::Uuid::new_v4().to_string()[..8].to_owned()
        );

        // Rule expects source "expected.app"
        events
            .put_rule()
            .name(&rule_name)
            .event_pattern(r#"{"source":["expected.app"]}"#)
            .send()
            .await
            .unwrap();

        let target_arn = format!("arn:aws:sqs:us-east-1:000000000000:{queue_name}");
        events
            .put_targets()
            .rule(&rule_name)
            .targets(
                Target::builder()
                    .id("sqs-1")
                    .arn(&target_arn)
                    .build()
                    .unwrap(),
            )
            .send()
            .await
            .unwrap();

        // Send event with different source
        events
            .put_events()
            .entries(
                PutEventsRequestEntry::builder()
                    .source("different.app")
                    .detail_type("Test")
                    .detail(r"{}")
                    .build(),
            )
            .send()
            .await
            .unwrap();

        // SQS should have 0 messages (short poll)
        let recv = sqs
            .receive_message()
            .queue_url(&queue_url)
            .wait_time_seconds(1)
            .send()
            .await
            .unwrap();
        assert_eq!(recv.messages().len(), 0, "no message should be delivered");

        // Cleanup
        events
            .remove_targets()
            .rule(&rule_name)
            .ids("sqs-1")
            .send()
            .await
            .unwrap();
        events.delete_rule().name(&rule_name).send().await.unwrap();
        delete_test_queue(&sqs, &queue_url).await;
    }
}
