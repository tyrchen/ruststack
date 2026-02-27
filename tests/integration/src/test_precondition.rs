//! HTTP precondition integration tests (If-Match, If-None-Match).

#[cfg(test)]
mod tests {
    use aws_sdk_s3::primitives::ByteStream;

    use crate::{cleanup_bucket, create_test_bucket, s3_client};

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_get_with_matching_if_match() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "ifmatch").await;

        let put = client
            .put_object()
            .bucket(&bucket)
            .key("cond.txt")
            .body(ByteStream::from_static(b"data"))
            .send()
            .await
            .expect("put");

        let etag = put.e_tag().expect("etag").to_owned();

        // If-Match with correct etag should succeed.
        let result = client
            .get_object()
            .bucket(&bucket)
            .key("cond.txt")
            .if_match(&etag)
            .send()
            .await;
        assert!(result.is_ok(), "if-match with correct etag should succeed");

        cleanup_bucket(&client, &bucket).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_fail_with_mismatched_if_match() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "ifmismatch").await;

        client
            .put_object()
            .bucket(&bucket)
            .key("cond.txt")
            .body(ByteStream::from_static(b"data"))
            .send()
            .await
            .expect("put");

        let result = client
            .get_object()
            .bucket(&bucket)
            .key("cond.txt")
            .if_match("\"wrong-etag\"")
            .send()
            .await;
        assert!(result.is_err(), "if-match with wrong etag should fail");

        cleanup_bucket(&client, &bucket).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_get_with_if_none_match_different() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "ifnone").await;

        client
            .put_object()
            .bucket(&bucket)
            .key("cond.txt")
            .body(ByteStream::from_static(b"data"))
            .send()
            .await
            .expect("put");

        // If-None-Match with a different etag should succeed (return the object).
        let result = client
            .get_object()
            .bucket(&bucket)
            .key("cond.txt")
            .if_none_match("\"different-etag\"")
            .send()
            .await;
        assert!(
            result.is_ok(),
            "if-none-match with different etag should succeed"
        );

        cleanup_bucket(&client, &bucket).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_return_not_modified_with_if_none_match_same() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "notmod").await;

        let put = client
            .put_object()
            .bucket(&bucket)
            .key("cond.txt")
            .body(ByteStream::from_static(b"data"))
            .send()
            .await
            .expect("put");

        let etag = put.e_tag().expect("etag").to_owned();

        // If-None-Match with the same etag should return 304 (treated as error by SDK).
        let result = client
            .get_object()
            .bucket(&bucket)
            .key("cond.txt")
            .if_none_match(&etag)
            .send()
            .await;
        assert!(
            result.is_err(),
            "if-none-match with same etag should return 304"
        );

        cleanup_bucket(&client, &bucket).await;
    }
}
