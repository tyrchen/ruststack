//! Auto-generated from AWS S3 Smithy model. DO NOT EDIT.

use std::collections::HashMap;

use crate::request::StreamingBlob;

use crate::types::{
    ChecksumAlgorithm, ChecksumType, CompletedMultipartUpload, EncodingType, ObjectCannedACL,
    ObjectLockLegalHoldStatus, ObjectLockMode, RequestPayer, ServerSideEncryption, StorageClass,
};

/// S3 AbortMultipartUploadInput.
#[derive(Debug, Clone, Default)]
pub struct AbortMultipartUploadInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP header: `x-amz-if-match-initiated-time`.
    pub if_match_initiated_time: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP query: `uploadId`.
    pub upload_id: String,
}

/// S3 CompleteMultipartUploadInput.
#[derive(Debug, Clone, Default)]
pub struct CompleteMultipartUploadInput {
    /// HTTP label (URI path).
    pub bucket: String,
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
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP header: `If-Match`.
    pub if_match: Option<String>,
    /// HTTP header: `If-None-Match`.
    pub if_none_match: Option<String>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP header: `x-amz-mp-object-size`.
    pub mpu_object_size: Option<i64>,
    /// HTTP payload body.
    pub multipart_upload: Option<CompletedMultipartUpload>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP header: `x-amz-server-side-encryption-customer-algorithm`.
    pub sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key`.
    pub sse_customer_key: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key-MD5`.
    pub sse_customer_key_md5: Option<String>,
    /// HTTP query: `uploadId`.
    pub upload_id: String,
}

/// S3 CreateMultipartUploadInput.
#[derive(Debug, Clone, Default)]
pub struct CreateMultipartUploadInput {
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
    /// HTTP header: `x-amz-checksum-type`.
    pub checksum_type: Option<ChecksumType>,
    /// HTTP header: `Content-Disposition`.
    pub content_disposition: Option<String>,
    /// HTTP header: `Content-Encoding`.
    pub content_encoding: Option<String>,
    /// HTTP header: `Content-Language`.
    pub content_language: Option<String>,
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
}

/// S3 ListMultipartUploadsInput.
#[derive(Debug, Clone, Default)]
pub struct ListMultipartUploadsInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP query: `delimiter`.
    pub delimiter: Option<String>,
    /// HTTP query: `encoding-type`.
    pub encoding_type: Option<EncodingType>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP query: `key-marker`.
    pub key_marker: Option<String>,
    /// HTTP query: `max-uploads`.
    pub max_uploads: Option<i32>,
    /// HTTP query: `prefix`.
    pub prefix: Option<String>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP query: `upload-id-marker`.
    pub upload_id_marker: Option<String>,
}

/// S3 ListPartsInput.
#[derive(Debug, Clone, Default)]
pub struct ListPartsInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP query: `max-parts`.
    pub max_parts: Option<i32>,
    /// HTTP query: `part-number-marker`.
    pub part_number_marker: Option<String>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP header: `x-amz-server-side-encryption-customer-algorithm`.
    pub sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key`.
    pub sse_customer_key: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key-MD5`.
    pub sse_customer_key_md5: Option<String>,
    /// HTTP query: `uploadId`.
    pub upload_id: String,
}

/// S3 UploadPartCopyInput.
#[derive(Debug, Clone, Default)]
pub struct UploadPartCopyInput {
    /// HTTP label (URI path).
    pub bucket: String,
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
    /// HTTP header: `x-amz-copy-source-range`.
    pub copy_source_range: Option<String>,
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
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP query: `partNumber`.
    pub part_number: i32,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP header: `x-amz-server-side-encryption-customer-algorithm`.
    pub sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key`.
    pub sse_customer_key: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key-MD5`.
    pub sse_customer_key_md5: Option<String>,
    /// HTTP query: `uploadId`.
    pub upload_id: String,
}

/// S3 UploadPartInput.
#[derive(Debug, Clone, Default)]
pub struct UploadPartInput {
    /// HTTP payload body.
    pub body: Option<StreamingBlob>,
    /// HTTP label (URI path).
    pub bucket: String,
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
    /// HTTP header: `Content-Length`.
    pub content_length: Option<i64>,
    /// HTTP header: `Content-MD5`.
    pub content_md5: Option<String>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP label (URI path).
    pub key: String,
    /// HTTP query: `partNumber`.
    pub part_number: i32,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP header: `x-amz-server-side-encryption-customer-algorithm`.
    pub sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key`.
    pub sse_customer_key: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key-MD5`.
    pub sse_customer_key_md5: Option<String>,
    /// HTTP query: `uploadId`.
    pub upload_id: String,
}
