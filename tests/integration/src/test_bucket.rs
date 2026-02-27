//! Bucket CRUD integration tests.

#[cfg(test)]
mod tests {
    use crate::{cleanup_bucket, create_test_bucket, s3_client, test_bucket_name};

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_create_and_delete_bucket() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "create").await;

        let head = client.head_bucket().bucket(&bucket).send().await;
        assert!(head.is_ok(), "head_bucket should succeed");

        cleanup_bucket(&client, &bucket).await;

        let head = client.head_bucket().bucket(&bucket).send().await;
        assert!(head.is_err(), "head_bucket should fail after delete");
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_list_buckets() {
        let client = s3_client();
        let b1 = create_test_bucket(&client, "list1").await;
        let b2 = create_test_bucket(&client, "list2").await;

        let resp = client.list_buckets().send().await.expect("list_buckets");
        let names: Vec<&str> = resp.buckets().iter().filter_map(|b| b.name()).collect();

        assert!(names.contains(&b1.as_str()), "should contain {b1}");
        assert!(names.contains(&b2.as_str()), "should contain {b2}");

        cleanup_bucket(&client, &b1).await;
        cleanup_bucket(&client, &b2).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_get_bucket_location() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "location").await;

        let resp = client
            .get_bucket_location()
            .bucket(&bucket)
            .send()
            .await
            .expect("get_bucket_location");

        // us-east-1 is returned as None or Some("us-east-1") depending on the implementation.
        let loc = resp.location_constraint();
        // Either None or the region value is acceptable for us-east-1.
        tracing::info!(?loc, "bucket location");

        cleanup_bucket(&client, &bucket).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_reject_duplicate_bucket() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "dup").await;

        let result = client.create_bucket().bucket(&bucket).send().await;
        assert!(result.is_err(), "duplicate bucket should fail");

        cleanup_bucket(&client, &bucket).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_reject_delete_nonempty_bucket() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "nonempty").await;

        client
            .put_object()
            .bucket(&bucket)
            .key("file.txt")
            .body(aws_sdk_s3::primitives::ByteStream::from_static(b"data"))
            .send()
            .await
            .expect("put_object");

        let result = client.delete_bucket().bucket(&bucket).send().await;
        assert!(result.is_err(), "deleting nonempty bucket should fail");

        cleanup_bucket(&client, &bucket).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_reject_nonexistent_bucket_head() {
        let client = s3_client();
        let name = test_bucket_name("ghost");

        let result = client.head_bucket().bucket(&name).send().await;
        assert!(result.is_err(), "head on nonexistent bucket should fail");
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_recreate_bucket_after_delete() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "recreate").await;

        cleanup_bucket(&client, &bucket).await;

        // Should be able to recreate.
        client
            .create_bucket()
            .bucket(&bucket)
            .send()
            .await
            .expect("recreate bucket");

        let head = client.head_bucket().bucket(&bucket).send().await;
        assert!(head.is_ok());

        cleanup_bucket(&client, &bucket).await;
    }
}
