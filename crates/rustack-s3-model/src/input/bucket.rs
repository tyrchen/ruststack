//! Auto-generated from AWS S3 Smithy model. DO NOT EDIT.

use crate::types::{BucketCannedACL, CreateBucketConfiguration, ObjectOwnership};

/// S3 CreateBucketInput.
#[derive(Debug, Clone, Default)]
pub struct CreateBucketInput {
    /// HTTP header: `x-amz-acl`.
    pub acl: Option<BucketCannedACL>,
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP payload body.
    pub create_bucket_configuration: Option<CreateBucketConfiguration>,
    /// HTTP header: `x-amz-grant-full-control`.
    pub grant_full_control: Option<String>,
    /// HTTP header: `x-amz-grant-read`.
    pub grant_read: Option<String>,
    /// HTTP header: `x-amz-grant-read-acp`.
    pub grant_read_acp: Option<String>,
    /// HTTP header: `x-amz-grant-write`.
    pub grant_write: Option<String>,
    /// HTTP header: `x-amz-grant-write-acp`.
    pub grant_write_acp: Option<String>,
    /// HTTP header: `x-amz-bucket-object-lock-enabled`.
    pub object_lock_enabled_for_bucket: Option<bool>,
    /// HTTP header: `x-amz-object-ownership`.
    pub object_ownership: Option<ObjectOwnership>,
}

/// S3 DeleteBucketInput.
#[derive(Debug, Clone, Default)]
pub struct DeleteBucketInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetBucketLocationInput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketLocationInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 HeadBucketInput.
#[derive(Debug, Clone, Default)]
pub struct HeadBucketInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 ListBucketsInput.
#[derive(Debug, Clone, Default)]
pub struct ListBucketsInput {
    /// HTTP query: `bucket-region`.
    pub bucket_region: Option<String>,
    /// HTTP query: `continuation-token`.
    pub continuation_token: Option<String>,
    /// HTTP query: `max-buckets`.
    pub max_buckets: Option<i32>,
    /// HTTP query: `prefix`.
    pub prefix: Option<String>,
}
