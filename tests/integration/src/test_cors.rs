//! CORS configuration integration tests.

#[cfg(test)]
mod tests {
    use aws_sdk_s3::types::{CorsConfiguration, CorsRule};

    use crate::{cleanup_bucket, create_test_bucket, s3_client};

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_put_and_get_cors() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "cors").await;

        let rule = CorsRule::builder()
            .allowed_origins("https://example.com")
            .allowed_methods("GET")
            .allowed_methods("PUT")
            .allowed_headers("*")
            .max_age_seconds(3600)
            .build()
            .expect("build cors rule");

        let cors_config = CorsConfiguration::builder()
            .cors_rules(rule)
            .build()
            .expect("build cors config");

        client
            .put_bucket_cors()
            .bucket(&bucket)
            .cors_configuration(cors_config)
            .send()
            .await
            .expect("put_bucket_cors");

        let resp = client
            .get_bucket_cors()
            .bucket(&bucket)
            .send()
            .await
            .expect("get_bucket_cors");

        let rules = resp.cors_rules();
        assert_eq!(rules.len(), 1);
        assert!(
            rules[0]
                .allowed_origins()
                .contains(&"https://example.com".to_owned())
        );

        cleanup_bucket(&client, &bucket).await;
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_delete_cors() {
        let client = s3_client();
        let bucket = create_test_bucket(&client, "delcors").await;

        let rule = CorsRule::builder()
            .allowed_origins("*")
            .allowed_methods("GET")
            .build()
            .expect("build rule");

        client
            .put_bucket_cors()
            .bucket(&bucket)
            .cors_configuration(
                CorsConfiguration::builder()
                    .cors_rules(rule)
                    .build()
                    .expect("build config"),
            )
            .send()
            .await
            .expect("put cors");

        client
            .delete_bucket_cors()
            .bucket(&bucket)
            .send()
            .await
            .expect("delete cors");

        let result = client.get_bucket_cors().bucket(&bucket).send().await;
        // After delete, get_cors should return an error (no CORS config).
        assert!(result.is_err(), "get_cors after delete should fail");

        cleanup_bucket(&client, &bucket).await;
    }
}
