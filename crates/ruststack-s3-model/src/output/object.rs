//! Auto-generated from AWS S3 Smithy model. DO NOT EDIT.

use std::collections::HashMap;

use crate::request::StreamingBlob;

use crate::types::{
    ArchiveStatus, ChecksumType, CopyObjectResult, DeletedObject, Error, ObjectLockLegalHoldStatus,
    ObjectLockMode, ReplicationStatus, RequestCharged, ServerSideEncryption, StorageClass,
};

/// S3 CopyObjectOutput.
#[derive(Debug, Clone, Default)]
pub struct CopyObjectOutput {
    /// HTTP header: `x-amz-server-side-encryption-bucket-key-enabled`.
    pub bucket_key_enabled: Option<bool>,
    /// HTTP payload body.
    pub copy_object_result: Option<CopyObjectResult>,
    /// HTTP header: `x-amz-copy-source-version-id`.
    pub copy_source_version_id: Option<String>,
    /// HTTP header: `x-amz-expiration`.
    pub expiration: Option<String>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
    /// HTTP header: `x-amz-server-side-encryption-customer-algorithm`.
    pub sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key-MD5`.
    pub sse_customer_key_md5: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-context`.
    pub ssekms_encryption_context: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-aws-kms-key-id`.
    pub ssekms_key_id: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption`.
    pub server_side_encryption: Option<ServerSideEncryption>,
    /// HTTP header: `x-amz-version-id`.
    pub version_id: Option<String>,
}

/// S3 DeleteObjectOutput.
#[derive(Debug, Clone, Default)]
pub struct DeleteObjectOutput {
    /// HTTP header: `x-amz-delete-marker`.
    pub delete_marker: Option<bool>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
    /// HTTP header: `x-amz-version-id`.
    pub version_id: Option<String>,
}

/// S3 DeleteObjectsOutput.
#[derive(Debug, Clone, Default)]
pub struct DeleteObjectsOutput {
    pub deleted: Vec<DeletedObject>,
    pub errors: Vec<Error>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
}

/// S3 GetObjectOutput.
#[derive(Debug, Clone, Default)]
pub struct GetObjectOutput {
    /// HTTP header: `accept-ranges`.
    pub accept_ranges: Option<String>,
    /// HTTP payload body.
    pub body: Option<StreamingBlob>,
    /// HTTP header: `x-amz-server-side-encryption-bucket-key-enabled`.
    pub bucket_key_enabled: Option<bool>,
    /// HTTP header: `Cache-Control`.
    pub cache_control: Option<String>,
    /// HTTP header: `x-amz-checksum-crc32`.
    pub checksum_crc32: Option<String>,
    /// HTTP header: `x-amz-checksum-crc32c`.
    pub checksum_crc32c: Option<String>,
    /// HTTP header: `x-amz-checksum-crc64nvme`.
    pub checksum_crc64nvme: Option<String>,
    /// HTTP header: `x-amz-checksum-sha1`.
    pub checksum_sha1: Option<String>,
    /// HTTP header: `x-amz-checksum-sha256`.
    pub checksum_sha256: Option<String>,
    /// HTTP header: `x-amz-checksum-type`.
    pub checksum_type: Option<ChecksumType>,
    /// HTTP header: `Content-Disposition`.
    pub content_disposition: Option<String>,
    /// HTTP header: `Content-Encoding`.
    pub content_encoding: Option<String>,
    /// HTTP header: `Content-Language`.
    pub content_language: Option<String>,
    /// HTTP header: `Content-Length`.
    pub content_length: Option<i64>,
    /// HTTP header: `Content-Range`.
    pub content_range: Option<String>,
    /// HTTP header: `Content-Type`.
    pub content_type: Option<String>,
    /// HTTP header: `x-amz-delete-marker`.
    pub delete_marker: Option<bool>,
    /// HTTP header: `ETag`.
    pub e_tag: Option<String>,
    /// HTTP header: `x-amz-expiration`.
    pub expiration: Option<String>,
    /// HTTP header: `Expires`.
    pub expires: Option<String>,
    /// HTTP header: `Last-Modified`.
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP prefix headers: `x-amz-meta-`.
    pub metadata: HashMap<String, String>,
    /// HTTP header: `x-amz-missing-meta`.
    pub missing_meta: Option<i32>,
    /// HTTP header: `x-amz-object-lock-legal-hold`.
    pub object_lock_legal_hold_status: Option<ObjectLockLegalHoldStatus>,
    /// HTTP header: `x-amz-object-lock-mode`.
    pub object_lock_mode: Option<ObjectLockMode>,
    /// HTTP header: `x-amz-object-lock-retain-until-date`.
    pub object_lock_retain_until_date: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP header: `x-amz-mp-parts-count`.
    pub parts_count: Option<i32>,
    /// HTTP header: `x-amz-replication-status`.
    pub replication_status: Option<ReplicationStatus>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
    /// HTTP header: `x-amz-restore`.
    pub restore: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-algorithm`.
    pub sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key-MD5`.
    pub sse_customer_key_md5: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-aws-kms-key-id`.
    pub ssekms_key_id: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption`.
    pub server_side_encryption: Option<ServerSideEncryption>,
    /// HTTP header: `x-amz-storage-class`.
    pub storage_class: Option<StorageClass>,
    /// HTTP header: `x-amz-tagging-count`.
    pub tag_count: Option<i32>,
    /// HTTP header: `x-amz-version-id`.
    pub version_id: Option<String>,
    /// HTTP header: `x-amz-website-redirect-location`.
    pub website_redirect_location: Option<String>,
}

/// S3 HeadObjectOutput.
#[derive(Debug, Clone, Default)]
pub struct HeadObjectOutput {
    /// HTTP header: `accept-ranges`.
    pub accept_ranges: Option<String>,
    /// HTTP header: `x-amz-archive-status`.
    pub archive_status: Option<ArchiveStatus>,
    /// HTTP header: `x-amz-server-side-encryption-bucket-key-enabled`.
    pub bucket_key_enabled: Option<bool>,
    /// HTTP header: `Cache-Control`.
    pub cache_control: Option<String>,
    /// HTTP header: `x-amz-checksum-crc32`.
    pub checksum_crc32: Option<String>,
    /// HTTP header: `x-amz-checksum-crc32c`.
    pub checksum_crc32c: Option<String>,
    /// HTTP header: `x-amz-checksum-crc64nvme`.
    pub checksum_crc64nvme: Option<String>,
    /// HTTP header: `x-amz-checksum-sha1`.
    pub checksum_sha1: Option<String>,
    /// HTTP header: `x-amz-checksum-sha256`.
    pub checksum_sha256: Option<String>,
    /// HTTP header: `x-amz-checksum-type`.
    pub checksum_type: Option<ChecksumType>,
    /// HTTP header: `Content-Disposition`.
    pub content_disposition: Option<String>,
    /// HTTP header: `Content-Encoding`.
    pub content_encoding: Option<String>,
    /// HTTP header: `Content-Language`.
    pub content_language: Option<String>,
    /// HTTP header: `Content-Length`.
    pub content_length: Option<i64>,
    /// HTTP header: `Content-Range`.
    pub content_range: Option<String>,
    /// HTTP header: `Content-Type`.
    pub content_type: Option<String>,
    /// HTTP header: `x-amz-delete-marker`.
    pub delete_marker: Option<bool>,
    /// HTTP header: `ETag`.
    pub e_tag: Option<String>,
    /// HTTP header: `x-amz-expiration`.
    pub expiration: Option<String>,
    /// HTTP header: `Expires`.
    pub expires: Option<String>,
    /// HTTP header: `Last-Modified`.
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP prefix headers: `x-amz-meta-`.
    pub metadata: HashMap<String, String>,
    /// HTTP header: `x-amz-missing-meta`.
    pub missing_meta: Option<i32>,
    /// HTTP header: `x-amz-object-lock-legal-hold`.
    pub object_lock_legal_hold_status: Option<ObjectLockLegalHoldStatus>,
    /// HTTP header: `x-amz-object-lock-mode`.
    pub object_lock_mode: Option<ObjectLockMode>,
    /// HTTP header: `x-amz-object-lock-retain-until-date`.
    pub object_lock_retain_until_date: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP header: `x-amz-mp-parts-count`.
    pub parts_count: Option<i32>,
    /// HTTP header: `x-amz-replication-status`.
    pub replication_status: Option<ReplicationStatus>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
    /// HTTP header: `x-amz-restore`.
    pub restore: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-algorithm`.
    pub sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key-MD5`.
    pub sse_customer_key_md5: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-aws-kms-key-id`.
    pub ssekms_key_id: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption`.
    pub server_side_encryption: Option<ServerSideEncryption>,
    /// HTTP header: `x-amz-storage-class`.
    pub storage_class: Option<StorageClass>,
    /// HTTP header: `x-amz-tagging-count`.
    pub tag_count: Option<i32>,
    /// HTTP header: `x-amz-version-id`.
    pub version_id: Option<String>,
    /// HTTP header: `x-amz-website-redirect-location`.
    pub website_redirect_location: Option<String>,
}

/// S3 PutObjectOutput.
#[derive(Debug, Clone, Default)]
pub struct PutObjectOutput {
    /// HTTP header: `x-amz-server-side-encryption-bucket-key-enabled`.
    pub bucket_key_enabled: Option<bool>,
    /// HTTP header: `x-amz-checksum-crc32`.
    pub checksum_crc32: Option<String>,
    /// HTTP header: `x-amz-checksum-crc32c`.
    pub checksum_crc32c: Option<String>,
    /// HTTP header: `x-amz-checksum-crc64nvme`.
    pub checksum_crc64nvme: Option<String>,
    /// HTTP header: `x-amz-checksum-sha1`.
    pub checksum_sha1: Option<String>,
    /// HTTP header: `x-amz-checksum-sha256`.
    pub checksum_sha256: Option<String>,
    /// HTTP header: `x-amz-checksum-type`.
    pub checksum_type: Option<ChecksumType>,
    /// HTTP header: `ETag`.
    pub e_tag: Option<String>,
    /// HTTP header: `x-amz-expiration`.
    pub expiration: Option<String>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
    /// HTTP header: `x-amz-server-side-encryption-customer-algorithm`.
    pub sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key-MD5`.
    pub sse_customer_key_md5: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-context`.
    pub ssekms_encryption_context: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-aws-kms-key-id`.
    pub ssekms_key_id: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption`.
    pub server_side_encryption: Option<ServerSideEncryption>,
    /// HTTP header: `x-amz-object-size`.
    pub size: Option<i64>,
    /// HTTP header: `x-amz-version-id`.
    pub version_id: Option<String>,
}
