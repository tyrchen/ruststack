//! Auto-generated from AWS S3 Smithy model. DO NOT EDIT.

use crate::types::{
    ChecksumAlgorithm, ChecksumType, CommonPrefix, CopyPartResult, EncodingType, Initiator,
    MultipartUpload, Owner, Part, RequestCharged, ServerSideEncryption, StorageClass,
};

/// S3 AbortMultipartUploadOutput.
#[derive(Debug, Clone, Default)]
pub struct AbortMultipartUploadOutput {
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
}

/// S3 CompleteMultipartUploadOutput.
#[derive(Debug, Clone, Default)]
pub struct CompleteMultipartUploadOutput {
    pub bucket: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-bucket-key-enabled`.
    pub bucket_key_enabled: Option<bool>,
    pub checksum_crc32: Option<String>,
    pub checksum_crc32c: Option<String>,
    pub checksum_crc64nvme: Option<String>,
    pub checksum_sha1: Option<String>,
    pub checksum_sha256: Option<String>,
    pub checksum_type: Option<ChecksumType>,
    pub e_tag: Option<String>,
    /// HTTP header: `x-amz-expiration`.
    pub expiration: Option<String>,
    pub key: Option<String>,
    pub location: Option<String>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
    /// HTTP header: `x-amz-server-side-encryption-aws-kms-key-id`.
    pub ssekms_key_id: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption`.
    pub server_side_encryption: Option<ServerSideEncryption>,
    /// HTTP header: `x-amz-version-id`.
    pub version_id: Option<String>,
}

/// S3 CreateMultipartUploadOutput.
#[derive(Debug, Clone, Default)]
pub struct CreateMultipartUploadOutput {
    /// HTTP header: `x-amz-abort-date`.
    pub abort_date: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP header: `x-amz-abort-rule-id`.
    pub abort_rule_id: Option<String>,
    pub bucket: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-bucket-key-enabled`.
    pub bucket_key_enabled: Option<bool>,
    /// HTTP header: `x-amz-checksum-algorithm`.
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// HTTP header: `x-amz-checksum-type`.
    pub checksum_type: Option<ChecksumType>,
    pub key: Option<String>,
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
    pub upload_id: Option<String>,
}

/// S3 ListMultipartUploadsOutput.
#[derive(Debug, Clone, Default)]
pub struct ListMultipartUploadsOutput {
    pub bucket: Option<String>,
    pub common_prefixes: Vec<CommonPrefix>,
    pub delimiter: Option<String>,
    pub encoding_type: Option<EncodingType>,
    pub is_truncated: Option<bool>,
    pub key_marker: Option<String>,
    pub max_uploads: Option<i32>,
    pub next_key_marker: Option<String>,
    pub next_upload_id_marker: Option<String>,
    pub prefix: Option<String>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
    pub upload_id_marker: Option<String>,
    pub uploads: Vec<MultipartUpload>,
}

/// S3 ListPartsOutput.
#[derive(Debug, Clone, Default)]
pub struct ListPartsOutput {
    /// HTTP header: `x-amz-abort-date`.
    pub abort_date: Option<chrono::DateTime<chrono::Utc>>,
    /// HTTP header: `x-amz-abort-rule-id`.
    pub abort_rule_id: Option<String>,
    pub bucket: Option<String>,
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    pub checksum_type: Option<ChecksumType>,
    pub initiator: Option<Initiator>,
    pub is_truncated: Option<bool>,
    pub key: Option<String>,
    pub max_parts: Option<i32>,
    pub next_part_number_marker: Option<String>,
    pub owner: Option<Owner>,
    pub part_number_marker: Option<String>,
    pub parts: Vec<Part>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
    pub storage_class: Option<StorageClass>,
    pub upload_id: Option<String>,
}

/// S3 UploadPartCopyOutput.
#[derive(Debug, Clone, Default)]
pub struct UploadPartCopyOutput {
    /// HTTP header: `x-amz-server-side-encryption-bucket-key-enabled`.
    pub bucket_key_enabled: Option<bool>,
    /// HTTP payload body.
    pub copy_part_result: Option<CopyPartResult>,
    /// HTTP header: `x-amz-copy-source-version-id`.
    pub copy_source_version_id: Option<String>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
    /// HTTP header: `x-amz-server-side-encryption-customer-algorithm`.
    pub sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key-MD5`.
    pub sse_customer_key_md5: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-aws-kms-key-id`.
    pub ssekms_key_id: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption`.
    pub server_side_encryption: Option<ServerSideEncryption>,
}

/// S3 UploadPartOutput.
#[derive(Debug, Clone, Default)]
pub struct UploadPartOutput {
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
    /// HTTP header: `ETag`.
    pub e_tag: Option<String>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
    /// HTTP header: `x-amz-server-side-encryption-customer-algorithm`.
    pub sse_customer_algorithm: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-customer-key-MD5`.
    pub sse_customer_key_md5: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption-aws-kms-key-id`.
    pub ssekms_key_id: Option<String>,
    /// HTTP header: `x-amz-server-side-encryption`.
    pub server_side_encryption: Option<ServerSideEncryption>,
}
