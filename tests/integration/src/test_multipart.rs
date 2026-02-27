//! Multipart upload integration tests.

#[cfg(test)]
mod tests {
    use aws_sdk_s3::primitives::ByteStream;
    use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};

    use crate::{cleanup_bucket, create_test_bucket, s3_client};

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_complete_multipart_upload() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "mpu").await;

        // Create multipart upload.
        let create = client
            .create_multipart_upload()
            .bucket(&bucket)
            .key("multipart.bin")
            .send()
            .await
            .expect("create_multipart_upload");

        let upload_id = create.upload_id().expect("upload_id");

        // Upload 2 parts (minimum 5 MB each for real S3, but our server accepts smaller).
        let part1_data = vec![0xAAu8; 1024];
        let part1 = client
            .upload_part()
            .bucket(&bucket)
            .key("multipart.bin")
            .upload_id(upload_id)
            .part_number(1)
            .body(ByteStream::from(part1_data.clone()))
            .send()
            .await
            .expect("upload part 1");

        let part2_data = vec![0xBBu8; 1024];
        let part2 = client
            .upload_part()
            .bucket(&bucket)
            .key("multipart.bin")
            .upload_id(upload_id)
            .part_number(2)
            .body(ByteStream::from(part2_data.clone()))
            .send()
            .await
            .expect("upload part 2");

        // Complete the upload.
        let completed = CompletedMultipartUpload::builder()
            .parts(
                CompletedPart::builder()
                    .part_number(1)
                    .e_tag(part1.e_tag().unwrap_or_default())
                    .build(),
            )
            .parts(
                CompletedPart::builder()
                    .part_number(2)
                    .e_tag(part2.e_tag().unwrap_or_default())
                    .build(),
            )
            .build();

        let complete = client
            .complete_multipart_upload()
            .bucket(&bucket)
            .key("multipart.bin")
            .upload_id(upload_id)
            .multipart_upload(completed)
            .send()
            .await
            .expect("complete_multipart_upload");

        assert!(
            complete.e_tag().is_some(),
            "completed upload should have etag"
        );

        // Verify the object.
        let resp = client
            .get_object()
            .bucket(&bucket)
            .key("multipart.bin")
            .send()
            .await
            .expect("get multipart object");

        let data = resp.body.collect().await.expect("collect").into_bytes();
        assert_eq!(data.len(), 2048);
        assert!(data[..1024].iter().all(|&b| b == 0xAA));
        assert!(data[1024..].iter().all(|&b| b == 0xBB));

        cleanup_bucket(&client, &bucket).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_abort_multipart_upload() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "abort").await;

        let create = client
            .create_multipart_upload()
            .bucket(&bucket)
            .key("aborted.bin")
            .send()
            .await
            .expect("create");

        let upload_id = create.upload_id().expect("upload_id");

        // Upload a part.
        client
            .upload_part()
            .bucket(&bucket)
            .key("aborted.bin")
            .upload_id(upload_id)
            .part_number(1)
            .body(ByteStream::from_static(b"will be aborted"))
            .send()
            .await
            .expect("upload part");

        // Abort the upload.
        client
            .abort_multipart_upload()
            .bucket(&bucket)
            .key("aborted.bin")
            .upload_id(upload_id)
            .send()
            .await
            .expect("abort");

        // The key should not exist.
        let result = client
            .get_object()
            .bucket(&bucket)
            .key("aborted.bin")
            .send()
            .await;
        assert!(result.is_err(), "aborted upload key should not exist");

        cleanup_bucket(&client, &bucket).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_list_multipart_uploads() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "listmpu").await;

        // Create two multipart uploads.
        let c1 = client
            .create_multipart_upload()
            .bucket(&bucket)
            .key("file1.bin")
            .send()
            .await
            .expect("create 1");
        let c2 = client
            .create_multipart_upload()
            .bucket(&bucket)
            .key("file2.bin")
            .send()
            .await
            .expect("create 2");

        let resp = client
            .list_multipart_uploads()
            .bucket(&bucket)
            .send()
            .await
            .expect("list_multipart_uploads");

        let uploads = resp.uploads();
        assert!(uploads.len() >= 2, "should have at least 2 uploads");

        // Clean up.
        client
            .abort_multipart_upload()
            .bucket(&bucket)
            .key("file1.bin")
            .upload_id(c1.upload_id().unwrap())
            .send()
            .await
            .ok();
        client
            .abort_multipart_upload()
            .bucket(&bucket)
            .key("file2.bin")
            .upload_id(c2.upload_id().unwrap())
            .send()
            .await
            .ok();

        cleanup_bucket(&client, &bucket).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_list_parts() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "listparts").await;

        let create = client
            .create_multipart_upload()
            .bucket(&bucket)
            .key("parts.bin")
            .send()
            .await
            .expect("create");

        let upload_id = create.upload_id().expect("upload_id");

        for i in 1..=3_i32 {
            let fill_byte = u8::try_from(i).expect("part number fits in u8");
            client
                .upload_part()
                .bucket(&bucket)
                .key("parts.bin")
                .upload_id(upload_id)
                .part_number(i)
                .body(ByteStream::from(vec![fill_byte; 512]))
                .send()
                .await
                .unwrap_or_else(|e| panic!("upload part {i}: {e}"));
        }

        let resp = client
            .list_parts()
            .bucket(&bucket)
            .key("parts.bin")
            .upload_id(upload_id)
            .send()
            .await
            .expect("list_parts");

        assert_eq!(resp.parts().len(), 3);

        // Cleanup.
        client
            .abort_multipart_upload()
            .bucket(&bucket)
            .key("parts.bin")
            .upload_id(upload_id)
            .send()
            .await
            .ok();
        cleanup_bucket(&client, &bucket).await;
    }
}
