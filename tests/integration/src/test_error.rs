//! Error handling integration tests.

#[cfg(test)]
mod tests {
    use aws_sdk_s3::primitives::ByteStream;

    use crate::{cleanup_bucket, create_test_bucket, s3_client, test_bucket_name};

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_return_no_such_bucket_on_put() {
        let client = s3_client();
        let bucket = test_bucket_name("ghost");

        let result = client
            .put_object()
            .bucket(&bucket)
            .key("file.txt")
            .body(ByteStream::from_static(b"data"))
            .send()
            .await;

        assert!(result.is_err(), "put to nonexistent bucket should fail");
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_return_no_such_key_on_get() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "nokey").await;

        let result = client
            .get_object()
            .bucket(&bucket)
            .key("nonexistent.txt")
            .send()
            .await;

        assert!(result.is_err(), "get nonexistent key should fail");

        cleanup_bucket(&client, &bucket).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_return_no_such_bucket_on_get() {
        let client = s3_client();
        let bucket = test_bucket_name("nosuch");

        let result = client
            .get_object()
            .bucket(&bucket)
            .key("file.txt")
            .send()
            .await;

        assert!(result.is_err(), "get from nonexistent bucket should fail");
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_return_no_such_bucket_on_list() {
        let client = s3_client();
        let bucket = test_bucket_name("nolist");

        let result = client.list_objects_v2().bucket(&bucket).send().await;

        assert!(result.is_err(), "list on nonexistent bucket should fail");
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_return_bucket_not_empty_on_delete() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "notempty").await;

        client
            .put_object()
            .bucket(&bucket)
            .key("blocker.txt")
            .body(ByteStream::from_static(b"blocks delete"))
            .send()
            .await
            .expect("put");

        let result = client.delete_bucket().bucket(&bucket).send().await;
        assert!(result.is_err(), "delete nonempty bucket should fail");

        cleanup_bucket(&client, &bucket).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_return_no_such_upload_on_complete() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "noupload").await;

        let completed = aws_sdk_s3::types::CompletedMultipartUpload::builder().build();

        let result = client
            .complete_multipart_upload()
            .bucket(&bucket)
            .key("file.bin")
            .upload_id("nonexistent-upload-id")
            .multipart_upload(completed)
            .send()
            .await;

        assert!(result.is_err(), "complete with bogus upload_id should fail");

        cleanup_bucket(&client, &bucket).await;
    }
}
