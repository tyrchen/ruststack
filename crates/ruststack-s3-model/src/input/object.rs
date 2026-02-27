//! Auto-generated from AWS S3 Smithy model. DO NOT EDIT.

use std::collections::HashMap;

use crate::request::StreamingBlob;

use crate::types::{
    ChecksumAlgorithm, ChecksumMode, Delete, MetadataDirective, ObjectCannedACL,
    ObjectLockLegalHoldStatus, ObjectLockMode, RequestPayer, ServerSideEncryption, StorageClass,
    TaggingDirective,
};

/// S3 CopyObjectInput.
#[derive(Debug, Clone, Default)]
pub struct CopyObjectInput {
    /// HTTP header: `x-amz-acl`.
    pub acl: Option<ObjectCannedACL>,
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-server-side-encryption-bucket-key-enabled`.
    pub bucket_key_enabled: Option<bool>,
    /// HTTP header: `Cache-Control`.
    pub cache_control: Option<String>,
    /// HTTP header: `x-amz-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `Content-Disposition`.
    pub content_disposition: Option<String>,
    /// HTTP header: `Content-Encoding`.
    pub content_encoding: Option<String>,
    /// HTTP header: `Content-Language`.
    pub content_language: Option<String>,
    /// HTTP header: `Content-Type`.
    pub content_type: Option<String>,
    /// HTTP header: `x-amz-copy-source`.
    pub copy_source: String,
    /// HTTP header: `x-amz-copy-source-if-match`.
    pub copy_source_if_match: Option<String>,
    /// HTTP header: `x-amz-copy-source-if-modified-since`.
    pub copy_source_if_modified_since: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP header: `x-amz-copy-source-if-none-match`.
    pub copy_source_if_none_match: Option<String>,
    /// HTTP header: `x-amz-copy-source-if-unmodified-since`.
    pub copy_source_if_unmodified_since: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP header: `x-amz-copy-source-server-side-encryption-customer-algorithm`.
    pub copy_source_sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-copy-source-server-side-encryption-customer-key`.
    pub copy_source_sse_customer_key: Option<String>,
    /// HTTP header: `x-amz-copy-source-server-side-encryption-customer-key-MD5`.
    pub copy_source_sse_customer_key_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP header: `x-amz-source-expected-bucket-owner`.
    pub expected_source_bucket_owner: Option<String>,
    /// HTTP header: `Expires`.
    pub expires: Option<String>,
    /// HTTP header: `x-amz-grant-full-control`.
    pub grant_full_control: Option<String>,
    /// HTTP header: `x-amz-grant-read`.
    pub grant_read: Option<String>,
    /// HTTP header: `x-amz-grant-read-acp`.
    pub grant_read_acp: Option<String>,
    /// HTTP header: `x-amz-grant-write-acp`.
    pub grant_write_acp: Option<String>,
    /// HTTP header: `If-Match`.
    pub if_match: Option<String>,
    /// HTTP header: `If-None-Match`.
    pub if_none_match: Option<String>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP prefix headers: `x-amz-meta-`.
    pub metadata: HashMap<String, String>,
    /// HTTP header: `x-amz-metadata-directive`.
    pub metadata_directive: Option<MetadataDirective>,
    /// HTTP header: `x-amz-object-lock-legal-hold`.
    pub object_lock_legal_hold_status: Option<ObjectLockLegalHoldStatus>,
    /// HTTP header: `x-amz-object-lock-mode`.
    pub object_lock_mode: Option<ObjectLockMode>,
    /// HTTP header: `x-amz-object-lock-retain-until-date`.
    pub object_lock_retain_until_date: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP header: `x-amz-server-side-encryption-customer-algorithm`.
    pub sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key`.
    pub sse_customer_key: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key-MD5`.
    pub sse_customer_key_md5: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-context`.
    pub ssekms_encryption_context: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-aws-kms-key-id`.
    pub ssekms_key_id: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption`.
    pub server_side_encryption: Option<ServerSideEncryption>,
    /// HTTP header: `x-amz-storage-class`.
    pub storage_class: Option<StorageClass>,
    /// HTTP header: `x-amz-tagging`.
    pub tagging: Option<String>,
    /// HTTP header: `x-amz-tagging-directive`.
    pub tagging_directive: Option<TaggingDirective>,
    /// HTTP header: `x-amz-website-redirect-location`.
    pub website_redirect_location: Option<String>,
}

/// S3 DeleteObjectInput.
#[derive(Debug, Clone, Default)]
pub struct DeleteObjectInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-bypass-governance-retention`.
    pub bypass_governance_retention: Option<bool>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP header: `If-Match`.
    pub if_match: Option<String>,
    /// HTTP header: `x-amz-if-match-last-modified-time`.
    pub if_match_last_modified_time: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP header: `x-amz-if-match-size`.
    pub if_match_size: Option<i64>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP header: `x-amz-mfa`.
    pub mfa: Option<String>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP query: `versionId`.
    pub version_id: Option<String>,
}

/// S3 DeleteObjectsInput.
#[derive(Debug, Clone, Default)]
pub struct DeleteObjectsInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-bypass-governance-retention`.
    pub bypass_governance_retention: Option<bool>,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP payload body.
    pub delete: Delete,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP header: `x-amz-mfa`.
    pub mfa: Option<String>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
}

/// S3 GetObjectInput.
#[derive(Debug, Clone, Default)]
pub struct GetObjectInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-checksum-mode`.
    pub checksum_mode: Option<ChecksumMode>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP header: `If-Match`.
    pub if_match: Option<String>,
    /// HTTP header: `If-Modified-Since`.
    pub if_modified_since: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP header: `If-None-Match`.
    pub if_none_match: Option<String>,
    /// HTTP header: `If-Unmodified-Since`.
    pub if_unmodified_since: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP query: `partNumber`.
    pub part_number: Option<i32>,
    /// HTTP header: `Range`.
    pub range: Option<String>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP query: `response-cache-control`.
    pub response_cache_control: Option<String>,
    /// HTTP query: `response-content-disposition`.
    pub response_content_disposition: Option<String>,
    /// HTTP query: `response-content-encoding`.
    pub response_content_encoding: Option<String>,
    /// HTTP query: `response-content-language`.
    pub response_content_language: Option<String>,
    /// HTTP query: `response-content-type`.
    pub response_content_type: Option<String>,
    /// HTTP query: `response-expires`.
    pub response_expires: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP header: `x-amz-server-side-encryption-customer-algorithm`.
    pub sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key`.
    pub sse_customer_key: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key-MD5`.
    pub sse_customer_key_md5: Option<String>,
    /// HTTP query: `versionId`.
    pub version_id: Option<String>,
}

/// S3 HeadObjectInput.
#[derive(Debug, Clone, Default)]
pub struct HeadObjectInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-checksum-mode`.
    pub checksum_mode: Option<ChecksumMode>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP header: `If-Match`.
    pub if_match: Option<String>,
    /// HTTP header: `If-Modified-Since`.
    pub if_modified_since: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP header: `If-None-Match`.
    pub if_none_match: Option<String>,
    /// HTTP header: `If-Unmodified-Since`.
    pub if_unmodified_since: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP query: `partNumber`.
    pub part_number: Option<i32>,
    /// HTTP header: `Range`.
    pub range: Option<String>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP query: `response-cache-control`.
    pub response_cache_control: Option<String>,
    /// HTTP query: `response-content-disposition`.
    pub response_content_disposition: Option<String>,
    /// HTTP query: `response-content-encoding`.
    pub response_content_encoding: Option<String>,
    /// HTTP query: `response-content-language`.
    pub response_content_language: Option<String>,
    /// HTTP query: `response-content-type`.
    pub response_content_type: Option<String>,
    /// HTTP query: `response-expires`.
    pub response_expires: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP header: `x-amz-server-side-encryption-customer-algorithm`.
    pub sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key`.
    pub sse_customer_key: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key-MD5`.
    pub sse_customer_key_md5: Option<String>,
    /// HTTP query: `versionId`.
    pub version_id: Option<String>,
}

/// S3 PutObjectInput.
#[derive(Debug, Clone, Default)]
pub struct PutObjectInput {
    /// HTTP header: `x-amz-acl`.
    pub acl: Option<ObjectCannedACL>,
    /// HTTP payload body.
    pub body: Option<StreamingBlob>,
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-server-side-encryption-bucket-key-enabled`.
    pub bucket_key_enabled: Option<bool>,
    /// HTTP header: `Cache-Control`.
    pub cache_control: Option<String>,
    /// HTTP header: `x-amz-sdk-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
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
    /// HTTP header: `Content-Disposition`.
    pub content_disposition: Option<String>,
    /// HTTP header: `Content-Encoding`.
    pub content_encoding: Option<String>,
    /// HTTP header: `Content-Language`.
    pub content_language: Option<String>,
    /// HTTP header: `Content-Length`.
    pub content_length: Option<i64>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `Content-Type`.
    pub content_type: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP header: `Expires`.
    pub expires: Option<String>,
    /// HTTP header: `x-amz-grant-full-control`.
    pub grant_full_control: Option<String>,
    /// HTTP header: `x-amz-grant-read`.
    pub grant_read: Option<String>,
    /// HTTP header: `x-amz-grant-read-acp`.
    pub grant_read_acp: Option<String>,
    /// HTTP header: `x-amz-grant-write-acp`.
    pub grant_write_acp: Option<String>,
    /// HTTP header: `If-Match`.
    pub if_match: Option<String>,
    /// HTTP header: `If-None-Match`.
    pub if_none_match: Option<String>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP prefix headers: `x-amz-meta-`.
    pub metadata: HashMap<String, String>,
    /// HTTP header: `x-amz-object-lock-legal-hold`.
    pub object_lock_legal_hold_status: Option<ObjectLockLegalHoldStatus>,
    /// HTTP header: `x-amz-object-lock-mode`.
    pub object_lock_mode: Option<ObjectLockMode>,
    /// HTTP header: `x-amz-object-lock-retain-until-date`.
    pub object_lock_retain_until_date: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP header: `x-amz-server-side-encryption-customer-algorithm`.
    pub sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key`.
    pub sse_customer_key: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key-MD5`.
    pub sse_customer_key_md5: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-context`.
    pub ssekms_encryption_context: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-aws-kms-key-id`.
    pub ssekms_key_id: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption`.
    pub server_side_encryption: Option<ServerSideEncryption>,
    /// HTTP header: `x-amz-storage-class`.
    pub storage_class: Option<StorageClass>,
    /// HTTP header: `x-amz-tagging`.
    pub tagging: Option<String>,
    /// HTTP header: `x-amz-website-redirect-location`.
    pub website_redirect_location: Option<String>,
    /// HTTP header: `x-amz-write-offset-bytes`.
    pub write_offset_bytes: Option<i64>,
}
