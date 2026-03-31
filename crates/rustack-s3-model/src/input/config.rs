//! Auto-generated from AWS S3 Smithy model. DO NOT EDIT.

use crate::types::{
    AccelerateConfiguration, AccessControlPolicy, BucketCannedACL, BucketLifecycleConfiguration,
    BucketLoggingStatus, CORSConfiguration, ChecksumAlgorithm, NotificationConfiguration,
    ObjectAttributes, ObjectCannedACL, ObjectLockConfiguration, ObjectLockLegalHold,
    ObjectLockRetention, OwnershipControls, PublicAccessBlockConfiguration, RequestPayer,
    RequestPaymentConfiguration, ServerSideEncryptionConfiguration, Tagging,
    TransitionDefaultMinimumObjectSize, VersioningConfiguration, WebsiteConfiguration,
};

/// S3 DeleteBucketCorsInput.
#[derive(Debug, Clone, Default)]
pub struct DeleteBucketCorsInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 DeleteBucketEncryptionInput.
#[derive(Debug, Clone, Default)]
pub struct DeleteBucketEncryptionInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 DeleteBucketLifecycleInput.
#[derive(Debug, Clone, Default)]
pub struct DeleteBucketLifecycleInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 DeleteBucketOwnershipControlsInput.
#[derive(Debug, Clone, Default)]
pub struct DeleteBucketOwnershipControlsInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 DeleteBucketPolicyInput.
#[derive(Debug, Clone, Default)]
pub struct DeleteBucketPolicyInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 DeleteBucketTaggingInput.
#[derive(Debug, Clone, Default)]
pub struct DeleteBucketTaggingInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 DeleteBucketWebsiteInput.
#[derive(Debug, Clone, Default)]
pub struct DeleteBucketWebsiteInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 DeleteObjectTaggingInput.
#[derive(Debug, Clone, Default)]
pub struct DeleteObjectTaggingInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP query: `versionId`.
    pub version_id: Option<String>,
}

/// S3 DeletePublicAccessBlockInput.
#[derive(Debug, Clone, Default)]
pub struct DeletePublicAccessBlockInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetBucketAccelerateConfigurationInput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketAccelerateConfigurationInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
}

/// S3 GetBucketAclInput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketAclInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetBucketCorsInput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketCorsInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetBucketEncryptionInput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketEncryptionInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetBucketLifecycleConfigurationInput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketLifecycleConfigurationInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetBucketLoggingInput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketLoggingInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetBucketNotificationConfigurationInput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketNotificationConfigurationInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetBucketOwnershipControlsInput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketOwnershipControlsInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetBucketPolicyInput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketPolicyInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetBucketPolicyStatusInput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketPolicyStatusInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetBucketRequestPaymentInput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketRequestPaymentInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetBucketTaggingInput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketTaggingInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetBucketVersioningInput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketVersioningInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetBucketWebsiteInput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketWebsiteInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetObjectAclInput.
#[derive(Debug, Clone, Default)]
pub struct GetObjectAclInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP query: `versionId`.
    pub version_id: Option<String>,
}

/// S3 GetObjectAttributesInput.
#[derive(Debug, Clone, Default)]
pub struct GetObjectAttributesInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP header: `x-amz-max-parts`.
    pub max_parts: Option<i32>,
    /// HTTP header: `x-amz-object-attributes`.
    pub object_attributes: Vec<ObjectAttributes>,
    /// HTTP header: `x-amz-part-number-marker`.
    pub part_number_marker: Option<String>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP header: `x-amz-server-side-encryption-customer-algorithm`.
    pub sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key`.
    pub sse_customer_key: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key-MD5`.
    pub sse_customer_key_md5: Option<String>,
    /// HTTP query: `versionId`.
    pub version_id: Option<String>,
}

/// S3 GetObjectLegalHoldInput.
#[derive(Debug, Clone, Default)]
pub struct GetObjectLegalHoldInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP query: `versionId`.
    pub version_id: Option<String>,
}

/// S3 GetObjectLockConfigurationInput.
#[derive(Debug, Clone, Default)]
pub struct GetObjectLockConfigurationInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 GetObjectRetentionInput.
#[derive(Debug, Clone, Default)]
pub struct GetObjectRetentionInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP query: `versionId`.
    pub version_id: Option<String>,
}

/// S3 GetObjectTaggingInput.
#[derive(Debug, Clone, Default)]
pub struct GetObjectTaggingInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP query: `versionId`.
    pub version_id: Option<String>,
}

/// S3 GetPublicAccessBlockInput.
#[derive(Debug, Clone, Default)]
pub struct GetPublicAccessBlockInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 PutBucketAccelerateConfigurationInput.
#[derive(Debug, Clone, Default)]
pub struct PutBucketAccelerateConfigurationInput {
    /// HTTP payload body.
    pub accelerate_configuration: AccelerateConfiguration,
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 PutBucketAclInput.
#[derive(Debug, Clone, Default)]
pub struct PutBucketAclInput {
    /// HTTP header: `x-amz-acl`.
    pub acl: Option<BucketCannedACL>,
    /// HTTP payload body.
    pub access_control_policy: Option<AccessControlPolicy>,
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
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
}

/// S3 PutBucketCorsInput.
#[derive(Debug, Clone, Default)]
pub struct PutBucketCorsInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP payload body.
    pub cors_configuration: CORSConfiguration,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 PutBucketEncryptionInput.
#[derive(Debug, Clone, Default)]
pub struct PutBucketEncryptionInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP payload body.
    pub server_side_encryption_configuration: ServerSideEncryptionConfiguration,
}

/// S3 PutBucketLifecycleConfigurationInput.
#[derive(Debug, Clone, Default)]
pub struct PutBucketLifecycleConfigurationInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP payload body.
    pub lifecycle_configuration: Option<BucketLifecycleConfiguration>,
    /// HTTP header: `x-amz-transition-default-minimum-object-size`.
    pub transition_default_minimum_object_size: Option<TransitionDefaultMinimumObjectSize>,
}

/// S3 PutBucketLoggingInput.
#[derive(Debug, Clone, Default)]
pub struct PutBucketLoggingInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP payload body.
    pub bucket_logging_status: BucketLoggingStatus,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
}

/// S3 PutBucketNotificationConfigurationInput.
#[derive(Debug, Clone, Default)]
pub struct PutBucketNotificationConfigurationInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP payload body.
    pub notification_configuration: NotificationConfiguration,
    /// HTTP header: `x-amz-skip-destination-validation`.
    pub skip_destination_validation: Option<bool>,
}

/// S3 PutBucketOwnershipControlsInput.
#[derive(Debug, Clone, Default)]
pub struct PutBucketOwnershipControlsInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP payload body.
    pub ownership_controls: OwnershipControls,
}

/// S3 PutBucketPolicyInput.
#[derive(Debug, Clone, Default)]
pub struct PutBucketPolicyInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `x-amz-confirm-remove-self-bucket-access`.
    pub confirm_remove_self_bucket_access: Option<bool>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP payload body.
    pub policy: String,
}

/// S3 PutBucketRequestPaymentInput.
#[derive(Debug, Clone, Default)]
pub struct PutBucketRequestPaymentInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP payload body.
    pub request_payment_configuration: RequestPaymentConfiguration,
}

/// S3 PutBucketTaggingInput.
#[derive(Debug, Clone, Default)]
pub struct PutBucketTaggingInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP payload body.
    pub tagging: Tagging,
}

/// S3 PutBucketVersioningInput.
#[derive(Debug, Clone, Default)]
pub struct PutBucketVersioningInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP header: `x-amz-mfa`.
    pub mfa: Option<String>,
    /// HTTP payload body.
    pub versioning_configuration: VersioningConfiguration,
}

/// S3 PutBucketWebsiteInput.
#[derive(Debug, Clone, Default)]
pub struct PutBucketWebsiteInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP payload body.
    pub website_configuration: WebsiteConfiguration,
}

/// S3 PutObjectAclInput.
#[derive(Debug, Clone, Default)]
pub struct PutObjectAclInput {
    /// HTTP header: `x-amz-acl`.
    pub acl: Option<ObjectCannedACL>,
    /// HTTP payload body.
    pub access_control_policy: Option<AccessControlPolicy>,
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
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
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP query: `versionId`.
    pub version_id: Option<String>,
}

/// S3 PutObjectLegalHoldInput.
#[derive(Debug, Clone, Default)]
pub struct PutObjectLegalHoldInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP payload body.
    pub legal_hold: Option<ObjectLockLegalHold>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP query: `versionId`.
    pub version_id: Option<String>,
}

/// S3 PutObjectLockConfigurationInput.
#[derive(Debug, Clone, Default)]
pub struct PutObjectLockConfigurationInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP payload body.
    pub object_lock_configuration: Option<ObjectLockConfiguration>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP header: `x-amz-bucket-object-lock-token`.
    pub token: Option<String>,
}

/// S3 PutObjectRetentionInput.
#[derive(Debug, Clone, Default)]
pub struct PutObjectRetentionInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-bypass-governance-retention`.
    pub bypass_governance_retention: Option<bool>,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP payload body.
    pub retention: Option<ObjectLockRetention>,
    /// HTTP query: `versionId`.
    pub version_id: Option<String>,
}

/// S3 PutObjectTaggingInput.
#[derive(Debug, Clone, Default)]
pub struct PutObjectTaggingInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP payload body.
    pub tagging: Tagging,
    /// HTTP query: `versionId`.
    pub version_id: Option<String>,
}

/// S3 PutPublicAccessBlockInput.
#[derive(Debug, Clone, Default)]
pub struct PutPublicAccessBlockInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP payload body.
    pub public_access_block_configuration: PublicAccessBlockConfiguration,
}
