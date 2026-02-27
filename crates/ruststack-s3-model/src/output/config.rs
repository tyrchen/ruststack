//! Auto-generated from AWS S3 Smithy model. DO NOT EDIT.

use crate::types::{
    BucketAccelerateStatus, BucketVersioningStatus, CORSRule, Checksum, ErrorDocument,
    GetObjectAttributesParts, Grant, IndexDocument, LifecycleRule, LoggingEnabled, MFADeleteStatus,
    NotificationConfiguration, ObjectLockConfiguration, ObjectLockLegalHold, ObjectLockRetention,
    Owner, OwnershipControls, Payer, PolicyStatus, PublicAccessBlockConfiguration,
    RedirectAllRequestsTo, RequestCharged, RoutingRule, ServerSideEncryptionConfiguration,
    StorageClass, Tag, TransitionDefaultMinimumObjectSize,
};

/// S3 DeleteObjectTaggingOutput.
#[derive(Debug, Clone, Default)]
pub struct DeleteObjectTaggingOutput {
    /// HTTP header: `x-amz-version-id`.
    pub version_id: Option<String>,
}

/// S3 GetBucketAccelerateConfigurationOutput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketAccelerateConfigurationOutput {
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
    pub status: Option<BucketAccelerateStatus>,
}

/// S3 GetBucketAclOutput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketAclOutput {
    pub grants: Vec<Grant>,
    pub owner: Option<Owner>,
}

/// S3 GetBucketCorsOutput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketCorsOutput {
    pub cors_rules: Vec<CORSRule>,
}

/// S3 GetBucketEncryptionOutput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketEncryptionOutput {
    /// HTTP payload body.
    pub server_side_encryption_configuration: Option<ServerSideEncryptionConfiguration>,
}

/// S3 GetBucketLifecycleConfigurationOutput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketLifecycleConfigurationOutput {
    pub rules: Vec<LifecycleRule>,
    /// HTTP header: `x-amz-transition-default-minimum-object-size`.
    pub transition_default_minimum_object_size: Option<TransitionDefaultMinimumObjectSize>,
}

/// S3 GetBucketLoggingOutput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketLoggingOutput {
    pub logging_enabled: Option<LoggingEnabled>,
}

/// S3 GetBucketNotificationConfigurationOutput.
///
/// Returns the notification configuration for a bucket. The fields mirror
/// the `NotificationConfiguration` structure from the AWS S3 model.
#[derive(Debug, Clone, Default)]
pub struct GetBucketNotificationConfigurationOutput {
    /// The notification configuration for the bucket.
    pub notification_configuration: Option<NotificationConfiguration>,
}

/// S3 GetBucketOwnershipControlsOutput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketOwnershipControlsOutput {
    /// HTTP payload body.
    pub ownership_controls: Option<OwnershipControls>,
}

/// S3 GetBucketPolicyOutput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketPolicyOutput {
    /// HTTP payload body.
    pub policy: Option<String>,
}

/// S3 GetBucketPolicyStatusOutput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketPolicyStatusOutput {
    /// HTTP payload body.
    pub policy_status: Option<PolicyStatus>,
}

/// S3 GetBucketRequestPaymentOutput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketRequestPaymentOutput {
    pub payer: Option<Payer>,
}

/// S3 GetBucketTaggingOutput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketTaggingOutput {
    pub tag_set: Vec<Tag>,
}

/// S3 GetBucketVersioningOutput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketVersioningOutput {
    pub mfa_delete: Option<MFADeleteStatus>,
    pub status: Option<BucketVersioningStatus>,
}

/// S3 GetBucketWebsiteOutput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketWebsiteOutput {
    pub error_document: Option<ErrorDocument>,
    pub index_document: Option<IndexDocument>,
    pub redirect_all_requests_to: Option<RedirectAllRequestsTo>,
    pub routing_rules: Vec<RoutingRule>,
}

/// S3 GetObjectAclOutput.
#[derive(Debug, Clone, Default)]
pub struct GetObjectAclOutput {
    pub grants: Vec<Grant>,
    pub owner: Option<Owner>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
}

/// S3 GetObjectAttributesOutput.
#[derive(Debug, Clone, Default)]
pub struct GetObjectAttributesOutput {
    pub checksum: Option<Checksum>,
    /// HTTP header: `x-amz-delete-marker`.
    pub delete_marker: Option<bool>,
    pub e_tag: Option<String>,
    /// HTTP header: `Last-Modified`.
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub object_parts: Option<GetObjectAttributesParts>,
    pub object_size: Option<i64>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
    pub storage_class: Option<StorageClass>,
    /// HTTP header: `x-amz-version-id`.
    pub version_id: Option<String>,
}

/// S3 GetObjectLegalHoldOutput.
#[derive(Debug, Clone, Default)]
pub struct GetObjectLegalHoldOutput {
    /// HTTP payload body.
    pub legal_hold: Option<ObjectLockLegalHold>,
}

/// S3 GetObjectLockConfigurationOutput.
#[derive(Debug, Clone, Default)]
pub struct GetObjectLockConfigurationOutput {
    /// HTTP payload body.
    pub object_lock_configuration: Option<ObjectLockConfiguration>,
}

/// S3 GetObjectRetentionOutput.
#[derive(Debug, Clone, Default)]
pub struct GetObjectRetentionOutput {
    /// HTTP payload body.
    pub retention: Option<ObjectLockRetention>,
}

/// S3 GetObjectTaggingOutput.
#[derive(Debug, Clone, Default)]
pub struct GetObjectTaggingOutput {
    pub tag_set: Vec<Tag>,
    /// HTTP header: `x-amz-version-id`.
    pub version_id: Option<String>,
}

/// S3 GetPublicAccessBlockOutput.
#[derive(Debug, Clone, Default)]
pub struct GetPublicAccessBlockOutput {
    /// HTTP payload body.
    pub public_access_block_configuration: Option<PublicAccessBlockConfiguration>,
}

/// S3 PutBucketLifecycleConfigurationOutput.
#[derive(Debug, Clone, Default)]
pub struct PutBucketLifecycleConfigurationOutput {
    /// HTTP header: `x-amz-transition-default-minimum-object-size`.
    pub transition_default_minimum_object_size: Option<TransitionDefaultMinimumObjectSize>,
}

/// S3 PutObjectAclOutput.
#[derive(Debug, Clone, Default)]
pub struct PutObjectAclOutput {
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
}

/// S3 PutObjectLegalHoldOutput.
#[derive(Debug, Clone, Default)]
pub struct PutObjectLegalHoldOutput {
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
}

/// S3 PutObjectLockConfigurationOutput.
#[derive(Debug, Clone, Default)]
pub struct PutObjectLockConfigurationOutput {
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
}

/// S3 PutObjectRetentionOutput.
#[derive(Debug, Clone, Default)]
pub struct PutObjectRetentionOutput {
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
}

/// S3 PutObjectTaggingOutput.
#[derive(Debug, Clone, Default)]
pub struct PutObjectTaggingOutput {
    /// HTTP header: `x-amz-version-id`.
    pub version_id: Option<String>,
}
