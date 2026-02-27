//! Integration tests for RustStack S3 server.
//!
//! These tests require a running RustStack S3 server at `localhost:4566`.
//! They are marked `#[ignore]` so they don't run during normal `cargo test`.
//!
//! Run them with:
//! ```text
//! cargo test -p ruststack-s3-integration -- --ignored
//! ```

use std::sync::Once;

use aws_sdk_s3::Client;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};

static INIT: Once = Once::new();

/// Initialize tracing (once).
fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
            )
            .with_test_writer()
            .init();
    });
}

/// Endpoint URL for the S3 server.
fn endpoint_url() -> String {
    std::env::var("S3_ENDPOINT_URL").unwrap_or_else(|_| "http://localhost:4566".to_owned())
}

/// Create a configured S3 client pointing at the local server.
#[must_use]
pub fn s3_client() -> Client {
    init_tracing();

    let creds = Credentials::new("test", "test", None, None, "integration-test");

    let config = aws_sdk_s3::config::Builder::new()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .credentials_provider(creds)
        .endpoint_url(endpoint_url())
        .force_path_style(true)
        .build();

    Client::from_conf(config)
}

/// Generate a unique bucket name for a test.
#[must_use]
pub fn test_bucket_name(prefix: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string()[..8].to_owned();
    format!("test-{prefix}-{id}")
}

/// Create a bucket and return its name. Caller is responsible for cleanup.
pub async fn create_test_bucket(client: &Client, prefix: &str) -> String {
    let name = test_bucket_name(prefix);
    client
        .create_bucket()
        .bucket(&name)
        .send()
        .await
        .unwrap_or_else(|e| panic!("failed to create bucket {name}: {e}"));
    name
}

/// Delete all objects in a bucket, then delete the bucket.
pub async fn cleanup_bucket(client: &Client, bucket: &str) {
    // List and delete all objects.
    let mut continuation_token = None;
    loop {
        let mut req = client.list_objects_v2().bucket(bucket);
        if let Some(token) = continuation_token.take() {
            req = req.continuation_token(token);
        }
        let Ok(resp) = req.send().await else {
            return; // Bucket may not exist.
        };

        for obj in resp.contents() {
            if let Some(key) = obj.key() {
                let _ = client.delete_object().bucket(bucket).key(key).send().await;
            }
        }

        if resp.is_truncated() == Some(true) {
            continuation_token = resp.next_continuation_token().map(ToOwned::to_owned);
        } else {
            break;
        }
    }

    // Delete any in-progress multipart uploads.
    if let Ok(uploads) = client.list_multipart_uploads().bucket(bucket).send().await {
        for upload in uploads.uploads() {
            if let (Some(key), Some(id)) = (upload.key(), upload.upload_id()) {
                let _ = client
                    .abort_multipart_upload()
                    .bucket(bucket)
                    .key(key)
                    .upload_id(id)
                    .send()
                    .await;
            }
        }
    }

    let _ = client.delete_bucket().bucket(bucket).send().await;
}

mod test_bucket;
mod test_cors;
mod test_error;
mod test_list;
mod test_multipart;
mod test_object;
mod test_precondition;
mod test_versioning;
