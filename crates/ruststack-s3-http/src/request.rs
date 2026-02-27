//! HTTP request to S3 Input struct deserialization.
//!
//! This module provides the [`FromS3Request`] trait and implementations for converting
//! raw HTTP request parts (headers, query parameters, URI labels, body) into typed S3
//! Input structs defined in `ruststack-s3-model`.
//!
//! Field extraction is guided by the doc comments on generated input struct fields:
//! - `HTTP header: x-amz-xxx` - Extract from request headers
//! - `HTTP query: name` - Extract from query parameters
//! - `HTTP label (URI path)` - From the bucket/key routing context
//! - `HTTP payload body` - From the request body (XML or raw bytes)
//! - `HTTP prefix headers: x-amz-meta-` - Collect all `x-amz-meta-*` headers

use std::collections::HashMap;
use std::str::FromStr;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use ruststack_s3_model::error::{S3Error, S3ErrorCode};
use ruststack_s3_model::request::StreamingBlob;
use ruststack_s3_model::types::{
    AccelerateConfiguration, AccessControlPolicy, BucketLifecycleConfiguration,
    BucketLoggingStatus, CORSConfiguration, CompletedMultipartUpload, CreateBucketConfiguration,
    Delete, NotificationConfiguration, ObjectLockConfiguration, ObjectLockLegalHold,
    ObjectLockRetention, OwnershipControls, PublicAccessBlockConfiguration,
    RequestPaymentConfiguration, ServerSideEncryptionConfiguration, Tagging,
    VersioningConfiguration, WebsiteConfiguration,
};
use ruststack_s3_xml::from_xml;

/// Trait for extracting an S3 input struct from HTTP request components.
///
/// Each S3 operation has a corresponding Input struct. Implementors of this trait
/// know how to populate that struct from the HTTP request parts.
pub trait FromS3Request: Sized {
    /// Extract the input from HTTP request parts.
    ///
    /// # Arguments
    /// - `parts` - The HTTP request head (method, URI, headers).
    /// - `bucket` - The resolved bucket name, if any.
    /// - `key` - The resolved object key, if any.
    /// - `query_params` - Parsed query parameters from the URI.
    /// - `body` - The raw request body bytes.
    ///
    /// # Errors
    ///
    /// Returns an `S3Error` if required fields are missing or field values
    /// cannot be parsed.
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error>;
}

// ---------------------------------------------------------------------------
// Helper functions for extracting typed values from HTTP request parts
// ---------------------------------------------------------------------------

/// Extract a header value as a string.
pub fn header_str(parts: &http::request::Parts, name: &str) -> Option<String> {
    parts
        .headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned)
}

/// Extract a header value and parse it into a type implementing `FromStr`.
pub fn header_parse<T: FromStr>(parts: &http::request::Parts, name: &str) -> Option<T> {
    parts
        .headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
}

/// Extract a header value and parse it as a boolean.
///
/// Recognizes "true" (case-insensitive) as `true`, everything else as `false`.
pub fn header_bool(parts: &http::request::Parts, name: &str) -> Option<bool> {
    parts
        .headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.eq_ignore_ascii_case("true"))
}

/// Extract a header value and parse it as an HTTP date timestamp.
///
/// Supports RFC 2822 and RFC 3339 / ISO 8601 formats.
pub fn header_timestamp(parts: &http::request::Parts, name: &str) -> Option<DateTime<Utc>> {
    let value = parts.headers.get(name)?.to_str().ok()?;
    parse_http_date(value)
}

/// Parse an HTTP date string into a `DateTime<Utc>`.
///
/// Tries multiple date formats commonly used in HTTP and AWS:
/// - ISO 8601 / RFC 3339 (e.g., `2024-01-15T10:30:00Z`)
/// - RFC 2822 (e.g., `Mon, 15 Jan 2024 10:30:00 GMT`)
fn parse_http_date(s: &str) -> Option<DateTime<Utc>> {
    // Try ISO 8601 / RFC 3339.
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    // Try RFC 2822.
    if let Ok(dt) = DateTime::parse_from_rfc2822(s) {
        return Some(dt.with_timezone(&Utc));
    }
    // Try common HTTP date formats.
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%a, %d %b %Y %H:%M:%S GMT") {
        return Some(dt.and_utc());
    }
    None
}

/// Get a query parameter value by name.
#[must_use]
pub fn query_param(params: &[(String, String)], name: &str) -> Option<String> {
    params
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.clone())
}

/// Get a query parameter and parse it into a type implementing `FromStr`.
#[must_use]
pub fn query_param_parse<T: FromStr>(params: &[(String, String)], name: &str) -> Option<T> {
    params
        .iter()
        .find(|(k, _)| k == name)
        .and_then(|(_, v)| v.parse().ok())
}

/// Collect all `x-amz-meta-*` headers into a metadata `HashMap`.
///
/// The key in the returned map is the portion of the header name after `x-amz-meta-`,
/// preserving the original case.
pub fn collect_metadata(parts: &http::request::Parts) -> HashMap<String, String> {
    let prefix = "x-amz-meta-";
    parts
        .headers
        .iter()
        .filter_map(|(name, value)| {
            let name_str = name.as_str();
            if let Some(meta_key) = name_str.strip_prefix(prefix) {
                let meta_value = value.to_str().ok()?;
                Some((meta_key.to_owned(), meta_value.to_owned()))
            } else {
                None
            }
        })
        .collect()
}

/// Require a bucket name from the routing context, returning an error if absent.
fn require_bucket(bucket: Option<&str>) -> Result<String, S3Error> {
    bucket.map(ToOwned::to_owned).ok_or_else(|| {
        S3Error::with_message(S3ErrorCode::InvalidRequest, "Bucket name is required")
    })
}

/// Require an object key from the routing context, returning an error if absent.
fn require_key(key: Option<&str>) -> Result<String, S3Error> {
    key.map(ToOwned::to_owned)
        .ok_or_else(|| S3Error::with_message(S3ErrorCode::InvalidRequest, "Object key is required"))
}

/// Extract a header value and convert it using `From<&str>` (for enum types).
fn header_enum<T>(parts: &http::request::Parts, name: &str) -> Option<T>
where
    T: for<'a> From<&'a str>,
{
    parts
        .headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(T::from)
}

/// Extract a comma-separated header into a vector of enum values.
fn header_enum_list<T>(parts: &http::request::Parts, name: &str) -> Vec<T>
where
    T: for<'a> From<&'a str>,
{
    parts
        .headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').map(|item| T::from(item.trim())).collect())
        .unwrap_or_default()
}

/// Parse an XML body into a typed value, returning an `S3Error` on failure.
fn parse_xml_body<T: ruststack_s3_xml::S3Deserialize>(body: &Bytes) -> Result<T, S3Error> {
    from_xml(body).map_err(|e| S3Error::malformed_xml(format!("Failed to parse XML body: {e}")))
}

// ---------------------------------------------------------------------------
// Macro to reduce boilerplate for simple bucket-only inputs
// ---------------------------------------------------------------------------

/// Implement `FromS3Request` for a struct that only has `bucket` and
/// optional `expected_bucket_owner` fields.
macro_rules! impl_bucket_only_input {
    ($ty:ty) => {
        impl FromS3Request for $ty {
            fn from_s3_request(
                parts: &http::request::Parts,
                bucket: Option<&str>,
                _key: Option<&str>,
                _query_params: &[(String, String)],
                _body: Bytes,
            ) -> Result<Self, S3Error> {
                Ok(Self {
                    bucket: require_bucket(bucket)?,
                    expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
                })
            }
        }
    };
}

/// Implement `FromS3Request` for a struct that has `bucket`, `key`,
/// optional `expected_bucket_owner`, and optional `version_id` query param.
macro_rules! impl_bucket_key_version_input {
    ($ty:ty) => {
        impl FromS3Request for $ty {
            #[allow(clippy::needless_update)]
            fn from_s3_request(
                parts: &http::request::Parts,
                bucket: Option<&str>,
                key: Option<&str>,
                query_params: &[(String, String)],
                _body: Bytes,
            ) -> Result<Self, S3Error> {
                Ok(Self {
                    bucket: require_bucket(bucket)?,
                    key: require_key(key)?,
                    expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
                    version_id: query_param(query_params, "versionId"),
                    ..Self::default()
                })
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Implementations for all Input types
// ---------------------------------------------------------------------------

#[allow(clippy::wildcard_imports)] // All 70+ input types are used via macro invocations below.
use ruststack_s3_model::input::*;

// --- Bucket operations (bucket-only) ---

impl_bucket_only_input!(DeleteBucketCorsInput);
impl_bucket_only_input!(DeleteBucketEncryptionInput);
impl_bucket_only_input!(DeleteBucketLifecycleInput);
impl_bucket_only_input!(DeleteBucketOwnershipControlsInput);
impl_bucket_only_input!(DeleteBucketPolicyInput);
impl_bucket_only_input!(DeleteBucketTaggingInput);
impl_bucket_only_input!(DeleteBucketWebsiteInput);
impl_bucket_only_input!(DeletePublicAccessBlockInput);
impl_bucket_only_input!(GetBucketCorsInput);
impl_bucket_only_input!(GetBucketEncryptionInput);
impl_bucket_only_input!(GetBucketLifecycleConfigurationInput);
impl_bucket_only_input!(GetBucketLoggingInput);
impl_bucket_only_input!(GetBucketNotificationConfigurationInput);
impl_bucket_only_input!(GetBucketOwnershipControlsInput);
impl_bucket_only_input!(GetBucketPolicyInput);
impl_bucket_only_input!(GetBucketPolicyStatusInput);
impl_bucket_only_input!(GetBucketRequestPaymentInput);
impl_bucket_only_input!(GetBucketTaggingInput);
impl_bucket_only_input!(GetBucketVersioningInput);
impl_bucket_only_input!(GetBucketWebsiteInput);
impl_bucket_only_input!(GetObjectLockConfigurationInput);
impl_bucket_only_input!(GetPublicAccessBlockInput);
impl_bucket_only_input!(GetBucketAclInput);

// --- Object operations (bucket + key + optional version) ---

impl_bucket_key_version_input!(DeleteObjectTaggingInput);
impl_bucket_key_version_input!(GetObjectTaggingInput);
impl_bucket_key_version_input!(GetObjectLegalHoldInput);
impl_bucket_key_version_input!(GetObjectRetentionInput);

impl FromS3Request for ListBucketsInput {
    fn from_s3_request(
        _parts: &http::request::Parts,
        _bucket: Option<&str>,
        _key: Option<&str>,
        query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket_region: query_param(query_params, "bucket-region"),
            continuation_token: query_param(query_params, "continuation-token"),
            max_buckets: query_param_parse(query_params, "max-buckets"),
            prefix: query_param(query_params, "prefix"),
        })
    }
}

impl FromS3Request for CreateBucketInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            acl: header_enum(parts, "x-amz-acl"),
            bucket: require_bucket(bucket)?,
            create_bucket_configuration: if body.is_empty() {
                None
            } else {
                Some(parse_xml_body::<CreateBucketConfiguration>(&body)?)
            },
            grant_full_control: header_str(parts, "x-amz-grant-full-control"),
            grant_read: header_str(parts, "x-amz-grant-read"),
            grant_read_acp: header_str(parts, "x-amz-grant-read-acp"),
            grant_write: header_str(parts, "x-amz-grant-write"),
            grant_write_acp: header_str(parts, "x-amz-grant-write-acp"),
            object_lock_enabled_for_bucket: header_bool(parts, "x-amz-bucket-object-lock-enabled"),
            object_ownership: header_enum(parts, "x-amz-object-ownership"),
        })
    }
}

impl FromS3Request for DeleteBucketInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
        })
    }
}

impl FromS3Request for GetBucketLocationInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
        })
    }
}

impl FromS3Request for HeadBucketInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
        })
    }
}

impl FromS3Request for GetBucketAccelerateConfigurationInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            request_payer: header_enum(parts, "x-amz-request-payer"),
        })
    }
}

impl FromS3Request for GetObjectAclInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            key: require_key(key)?,
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            request_payer: header_enum(parts, "x-amz-request-payer"),
            version_id: query_param(query_params, "versionId"),
        })
    }
}

impl FromS3Request for GetObjectAttributesInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            key: require_key(key)?,
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            max_parts: header_parse(parts, "x-amz-max-parts"),
            object_attributes: header_enum_list(parts, "x-amz-object-attributes"),
            part_number_marker: header_str(parts, "x-amz-part-number-marker"),
            request_payer: header_enum(parts, "x-amz-request-payer"),
            sse_customer_algorithm: header_str(
                parts,
                "x-amz-server-side-encryption-customer-algorithm",
            ),
            sse_customer_key: header_str(parts, "x-amz-server-side-encryption-customer-key"),
            sse_customer_key_md5: header_str(
                parts,
                "x-amz-server-side-encryption-customer-key-MD5",
            ),
            version_id: query_param(query_params, "versionId"),
        })
    }
}

impl FromS3Request for GetObjectInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            key: require_key(key)?,
            checksum_mode: header_enum(parts, "x-amz-checksum-mode"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            if_match: header_str(parts, "If-Match"),
            if_modified_since: header_timestamp(parts, "If-Modified-Since"),
            if_none_match: header_str(parts, "If-None-Match"),
            if_unmodified_since: header_timestamp(parts, "If-Unmodified-Since"),
            part_number: query_param_parse(query_params, "partNumber"),
            range: header_str(parts, "Range"),
            request_payer: header_enum(parts, "x-amz-request-payer"),
            response_cache_control: query_param(query_params, "response-cache-control"),
            response_content_disposition: query_param(query_params, "response-content-disposition"),
            response_content_encoding: query_param(query_params, "response-content-encoding"),
            response_content_language: query_param(query_params, "response-content-language"),
            response_content_type: query_param(query_params, "response-content-type"),
            response_expires: query_param(query_params, "response-expires")
                .and_then(|s| parse_http_date(&s)),
            sse_customer_algorithm: header_str(
                parts,
                "x-amz-server-side-encryption-customer-algorithm",
            ),
            sse_customer_key: header_str(parts, "x-amz-server-side-encryption-customer-key"),
            sse_customer_key_md5: header_str(
                parts,
                "x-amz-server-side-encryption-customer-key-MD5",
            ),
            version_id: query_param(query_params, "versionId"),
        })
    }
}

impl FromS3Request for HeadObjectInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            key: require_key(key)?,
            checksum_mode: header_enum(parts, "x-amz-checksum-mode"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            if_match: header_str(parts, "If-Match"),
            if_modified_since: header_timestamp(parts, "If-Modified-Since"),
            if_none_match: header_str(parts, "If-None-Match"),
            if_unmodified_since: header_timestamp(parts, "If-Unmodified-Since"),
            part_number: query_param_parse(query_params, "partNumber"),
            range: header_str(parts, "Range"),
            request_payer: header_enum(parts, "x-amz-request-payer"),
            response_cache_control: query_param(query_params, "response-cache-control"),
            response_content_disposition: query_param(query_params, "response-content-disposition"),
            response_content_encoding: query_param(query_params, "response-content-encoding"),
            response_content_language: query_param(query_params, "response-content-language"),
            response_content_type: query_param(query_params, "response-content-type"),
            response_expires: query_param(query_params, "response-expires")
                .and_then(|s| parse_http_date(&s)),
            sse_customer_algorithm: header_str(
                parts,
                "x-amz-server-side-encryption-customer-algorithm",
            ),
            sse_customer_key: header_str(parts, "x-amz-server-side-encryption-customer-key"),
            sse_customer_key_md5: header_str(
                parts,
                "x-amz-server-side-encryption-customer-key-MD5",
            ),
            version_id: query_param(query_params, "versionId"),
        })
    }
}

impl FromS3Request for PutObjectInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        let body_blob = if body.is_empty() {
            None
        } else {
            Some(StreamingBlob::new(body))
        };

        Ok(Self {
            acl: header_enum(parts, "x-amz-acl"),
            body: body_blob,
            bucket: require_bucket(bucket)?,
            bucket_key_enabled: header_bool(
                parts,
                "x-amz-server-side-encryption-bucket-key-enabled",
            ),
            cache_control: header_str(parts, "Cache-Control"),
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            checksum_crc32: header_str(parts, "x-amz-checksum-crc32"),
            checksum_crc32c: header_str(parts, "x-amz-checksum-crc32c"),
            checksum_crc64nvme: header_str(parts, "x-amz-checksum-crc64nvme"),
            checksum_sha1: header_str(parts, "x-amz-checksum-sha1"),
            checksum_sha256: header_str(parts, "x-amz-checksum-sha256"),
            content_disposition: header_str(parts, "Content-Disposition"),
            content_encoding: header_str(parts, "Content-Encoding"),
            content_language: header_str(parts, "Content-Language"),
            content_length: header_parse(parts, "Content-Length"),
            content_md5: header_str(parts, "Content-MD5"),
            content_type: header_str(parts, "Content-Type"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            expires: header_str(parts, "Expires"),
            grant_full_control: header_str(parts, "x-amz-grant-full-control"),
            grant_read: header_str(parts, "x-amz-grant-read"),
            grant_read_acp: header_str(parts, "x-amz-grant-read-acp"),
            grant_write_acp: header_str(parts, "x-amz-grant-write-acp"),
            if_match: header_str(parts, "If-Match"),
            if_none_match: header_str(parts, "If-None-Match"),
            key: require_key(key)?,
            metadata: collect_metadata(parts),
            object_lock_legal_hold_status: header_enum(parts, "x-amz-object-lock-legal-hold"),
            object_lock_mode: header_enum(parts, "x-amz-object-lock-mode"),
            object_lock_retain_until_date: header_timestamp(
                parts,
                "x-amz-object-lock-retain-until-date",
            ),
            request_payer: header_enum(parts, "x-amz-request-payer"),
            sse_customer_algorithm: header_str(
                parts,
                "x-amz-server-side-encryption-customer-algorithm",
            ),
            sse_customer_key: header_str(parts, "x-amz-server-side-encryption-customer-key"),
            sse_customer_key_md5: header_str(
                parts,
                "x-amz-server-side-encryption-customer-key-MD5",
            ),
            ssekms_encryption_context: header_str(parts, "x-amz-server-side-encryption-context"),
            ssekms_key_id: header_str(parts, "x-amz-server-side-encryption-aws-kms-key-id"),
            server_side_encryption: header_enum(parts, "x-amz-server-side-encryption"),
            storage_class: header_enum(parts, "x-amz-storage-class"),
            tagging: header_str(parts, "x-amz-tagging"),
            website_redirect_location: header_str(parts, "x-amz-website-redirect-location"),
            write_offset_bytes: header_parse(parts, "x-amz-write-offset-bytes"),
        })
    }
}

impl FromS3Request for CopyObjectInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        _query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        let copy_source = header_str(parts, "x-amz-copy-source").ok_or_else(|| {
            S3Error::with_message(
                S3ErrorCode::InvalidRequest,
                "x-amz-copy-source header is required for CopyObject",
            )
        })?;

        Ok(Self {
            acl: header_enum(parts, "x-amz-acl"),
            bucket: require_bucket(bucket)?,
            bucket_key_enabled: header_bool(
                parts,
                "x-amz-server-side-encryption-bucket-key-enabled",
            ),
            cache_control: header_str(parts, "Cache-Control"),
            checksum_algorithm: header_enum(parts, "x-amz-checksum-algorithm"),
            content_disposition: header_str(parts, "Content-Disposition"),
            content_encoding: header_str(parts, "Content-Encoding"),
            content_language: header_str(parts, "Content-Language"),
            content_type: header_str(parts, "Content-Type"),
            copy_source,
            copy_source_if_match: header_str(parts, "x-amz-copy-source-if-match"),
            copy_source_if_modified_since: header_timestamp(
                parts,
                "x-amz-copy-source-if-modified-since",
            ),
            copy_source_if_none_match: header_str(parts, "x-amz-copy-source-if-none-match"),
            copy_source_if_unmodified_since: header_timestamp(
                parts,
                "x-amz-copy-source-if-unmodified-since",
            ),
            copy_source_sse_customer_algorithm: header_str(
                parts,
                "x-amz-copy-source-server-side-encryption-customer-algorithm",
            ),
            copy_source_sse_customer_key: header_str(
                parts,
                "x-amz-copy-source-server-side-encryption-customer-key",
            ),
            copy_source_sse_customer_key_md5: header_str(
                parts,
                "x-amz-copy-source-server-side-encryption-customer-key-MD5",
            ),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            expected_source_bucket_owner: header_str(parts, "x-amz-source-expected-bucket-owner"),
            expires: header_str(parts, "Expires"),
            grant_full_control: header_str(parts, "x-amz-grant-full-control"),
            grant_read: header_str(parts, "x-amz-grant-read"),
            grant_read_acp: header_str(parts, "x-amz-grant-read-acp"),
            grant_write_acp: header_str(parts, "x-amz-grant-write-acp"),
            if_match: header_str(parts, "If-Match"),
            if_none_match: header_str(parts, "If-None-Match"),
            key: require_key(key)?,
            metadata: collect_metadata(parts),
            metadata_directive: header_enum(parts, "x-amz-metadata-directive"),
            object_lock_legal_hold_status: header_enum(parts, "x-amz-object-lock-legal-hold"),
            object_lock_mode: header_enum(parts, "x-amz-object-lock-mode"),
            object_lock_retain_until_date: header_timestamp(
                parts,
                "x-amz-object-lock-retain-until-date",
            ),
            request_payer: header_enum(parts, "x-amz-request-payer"),
            sse_customer_algorithm: header_str(
                parts,
                "x-amz-server-side-encryption-customer-algorithm",
            ),
            sse_customer_key: header_str(parts, "x-amz-server-side-encryption-customer-key"),
            sse_customer_key_md5: header_str(
                parts,
                "x-amz-server-side-encryption-customer-key-MD5",
            ),
            ssekms_encryption_context: header_str(parts, "x-amz-server-side-encryption-context"),
            ssekms_key_id: header_str(parts, "x-amz-server-side-encryption-aws-kms-key-id"),
            server_side_encryption: header_enum(parts, "x-amz-server-side-encryption"),
            storage_class: header_enum(parts, "x-amz-storage-class"),
            tagging: header_str(parts, "x-amz-tagging"),
            tagging_directive: header_enum(parts, "x-amz-tagging-directive"),
            website_redirect_location: header_str(parts, "x-amz-website-redirect-location"),
        })
    }
}

impl FromS3Request for DeleteObjectInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            bypass_governance_retention: header_bool(parts, "x-amz-bypass-governance-retention"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            if_match: header_str(parts, "If-Match"),
            if_match_last_modified_time: header_timestamp(
                parts,
                "x-amz-if-match-last-modified-time",
            ),
            if_match_size: header_parse(parts, "x-amz-if-match-size"),
            key: require_key(key)?,
            mfa: header_str(parts, "x-amz-mfa"),
            request_payer: header_enum(parts, "x-amz-request-payer"),
            version_id: query_param(query_params, "versionId"),
        })
    }
}

impl FromS3Request for DeleteObjectsInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            bypass_governance_retention: header_bool(parts, "x-amz-bypass-governance-retention"),
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            delete: parse_xml_body::<Delete>(&body)?,
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            mfa: header_str(parts, "x-amz-mfa"),
            request_payer: header_enum(parts, "x-amz-request-payer"),
        })
    }
}

// --- List operations ---

impl FromS3Request for ListObjectsInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            delimiter: query_param(query_params, "delimiter"),
            encoding_type: query_param(query_params, "encoding-type")
                .as_deref()
                .map(Into::into),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            marker: query_param(query_params, "marker"),
            max_keys: query_param_parse(query_params, "max-keys"),
            optional_object_attributes: header_enum_list(parts, "x-amz-optional-object-attributes"),
            prefix: query_param(query_params, "prefix"),
            request_payer: header_enum(parts, "x-amz-request-payer"),
        })
    }
}

impl FromS3Request for ListObjectsV2Input {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            continuation_token: query_param(query_params, "continuation-token"),
            delimiter: query_param(query_params, "delimiter"),
            encoding_type: query_param(query_params, "encoding-type")
                .as_deref()
                .map(Into::into),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            fetch_owner: query_param(query_params, "fetch-owner").map(|v| v == "true"),
            max_keys: query_param_parse(query_params, "max-keys"),
            optional_object_attributes: header_enum_list(parts, "x-amz-optional-object-attributes"),
            prefix: query_param(query_params, "prefix"),
            request_payer: header_enum(parts, "x-amz-request-payer"),
            start_after: query_param(query_params, "start-after"),
        })
    }
}

impl FromS3Request for ListObjectVersionsInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            delimiter: query_param(query_params, "delimiter"),
            encoding_type: query_param(query_params, "encoding-type")
                .as_deref()
                .map(Into::into),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            key_marker: query_param(query_params, "key-marker"),
            max_keys: query_param_parse(query_params, "max-keys"),
            optional_object_attributes: header_enum_list(parts, "x-amz-optional-object-attributes"),
            prefix: query_param(query_params, "prefix"),
            request_payer: header_enum(parts, "x-amz-request-payer"),
            version_id_marker: query_param(query_params, "version-id-marker"),
        })
    }
}

// --- Multipart operations ---

impl FromS3Request for CreateMultipartUploadInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        _query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            acl: header_enum(parts, "x-amz-acl"),
            bucket: require_bucket(bucket)?,
            bucket_key_enabled: header_bool(
                parts,
                "x-amz-server-side-encryption-bucket-key-enabled",
            ),
            cache_control: header_str(parts, "Cache-Control"),
            checksum_algorithm: header_enum(parts, "x-amz-checksum-algorithm"),
            checksum_type: header_enum(parts, "x-amz-checksum-type"),
            content_disposition: header_str(parts, "Content-Disposition"),
            content_encoding: header_str(parts, "Content-Encoding"),
            content_language: header_str(parts, "Content-Language"),
            content_type: header_str(parts, "Content-Type"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            expires: header_str(parts, "Expires"),
            grant_full_control: header_str(parts, "x-amz-grant-full-control"),
            grant_read: header_str(parts, "x-amz-grant-read"),
            grant_read_acp: header_str(parts, "x-amz-grant-read-acp"),
            grant_write_acp: header_str(parts, "x-amz-grant-write-acp"),
            key: require_key(key)?,
            metadata: collect_metadata(parts),
            object_lock_legal_hold_status: header_enum(parts, "x-amz-object-lock-legal-hold"),
            object_lock_mode: header_enum(parts, "x-amz-object-lock-mode"),
            object_lock_retain_until_date: header_timestamp(
                parts,
                "x-amz-object-lock-retain-until-date",
            ),
            request_payer: header_enum(parts, "x-amz-request-payer"),
            sse_customer_algorithm: header_str(
                parts,
                "x-amz-server-side-encryption-customer-algorithm",
            ),
            sse_customer_key: header_str(parts, "x-amz-server-side-encryption-customer-key"),
            sse_customer_key_md5: header_str(
                parts,
                "x-amz-server-side-encryption-customer-key-MD5",
            ),
            ssekms_encryption_context: header_str(parts, "x-amz-server-side-encryption-context"),
            ssekms_key_id: header_str(parts, "x-amz-server-side-encryption-aws-kms-key-id"),
            server_side_encryption: header_enum(parts, "x-amz-server-side-encryption"),
            storage_class: header_enum(parts, "x-amz-storage-class"),
            tagging: header_str(parts, "x-amz-tagging"),
            website_redirect_location: header_str(parts, "x-amz-website-redirect-location"),
        })
    }
}

impl FromS3Request for UploadPartInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        let part_number = query_param_parse(query_params, "partNumber").ok_or_else(|| {
            S3Error::with_message(
                S3ErrorCode::InvalidRequest,
                "partNumber query parameter is required",
            )
        })?;
        let upload_id = query_param(query_params, "uploadId").ok_or_else(|| {
            S3Error::with_message(
                S3ErrorCode::InvalidRequest,
                "uploadId query parameter is required",
            )
        })?;
        let body_blob = if body.is_empty() {
            None
        } else {
            Some(StreamingBlob::new(body))
        };

        Ok(Self {
            body: body_blob,
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            checksum_crc32: header_str(parts, "x-amz-checksum-crc32"),
            checksum_crc32c: header_str(parts, "x-amz-checksum-crc32c"),
            checksum_crc64nvme: header_str(parts, "x-amz-checksum-crc64nvme"),
            checksum_sha1: header_str(parts, "x-amz-checksum-sha1"),
            checksum_sha256: header_str(parts, "x-amz-checksum-sha256"),
            content_length: header_parse(parts, "Content-Length"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            key: require_key(key)?,
            part_number,
            request_payer: header_enum(parts, "x-amz-request-payer"),
            sse_customer_algorithm: header_str(
                parts,
                "x-amz-server-side-encryption-customer-algorithm",
            ),
            sse_customer_key: header_str(parts, "x-amz-server-side-encryption-customer-key"),
            sse_customer_key_md5: header_str(
                parts,
                "x-amz-server-side-encryption-customer-key-MD5",
            ),
            upload_id,
        })
    }
}

impl FromS3Request for UploadPartCopyInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        let copy_source = header_str(parts, "x-amz-copy-source").ok_or_else(|| {
            S3Error::with_message(
                S3ErrorCode::InvalidRequest,
                "x-amz-copy-source header is required for UploadPartCopy",
            )
        })?;
        let part_number = query_param_parse(query_params, "partNumber").ok_or_else(|| {
            S3Error::with_message(
                S3ErrorCode::InvalidRequest,
                "partNumber query parameter is required",
            )
        })?;
        let upload_id = query_param(query_params, "uploadId").ok_or_else(|| {
            S3Error::with_message(
                S3ErrorCode::InvalidRequest,
                "uploadId query parameter is required",
            )
        })?;

        Ok(Self {
            bucket: require_bucket(bucket)?,
            copy_source,
            copy_source_if_match: header_str(parts, "x-amz-copy-source-if-match"),
            copy_source_if_modified_since: header_timestamp(
                parts,
                "x-amz-copy-source-if-modified-since",
            ),
            copy_source_if_none_match: header_str(parts, "x-amz-copy-source-if-none-match"),
            copy_source_if_unmodified_since: header_timestamp(
                parts,
                "x-amz-copy-source-if-unmodified-since",
            ),
            copy_source_range: header_str(parts, "x-amz-copy-source-range"),
            copy_source_sse_customer_algorithm: header_str(
                parts,
                "x-amz-copy-source-server-side-encryption-customer-algorithm",
            ),
            copy_source_sse_customer_key: header_str(
                parts,
                "x-amz-copy-source-server-side-encryption-customer-key",
            ),
            copy_source_sse_customer_key_md5: header_str(
                parts,
                "x-amz-copy-source-server-side-encryption-customer-key-MD5",
            ),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            expected_source_bucket_owner: header_str(parts, "x-amz-source-expected-bucket-owner"),
            key: require_key(key)?,
            part_number,
            request_payer: header_enum(parts, "x-amz-request-payer"),
            sse_customer_algorithm: header_str(
                parts,
                "x-amz-server-side-encryption-customer-algorithm",
            ),
            sse_customer_key: header_str(parts, "x-amz-server-side-encryption-customer-key"),
            sse_customer_key_md5: header_str(
                parts,
                "x-amz-server-side-encryption-customer-key-MD5",
            ),
            upload_id,
        })
    }
}

impl FromS3Request for CompleteMultipartUploadInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        let upload_id = query_param(query_params, "uploadId").ok_or_else(|| {
            S3Error::with_message(
                S3ErrorCode::InvalidRequest,
                "uploadId query parameter is required",
            )
        })?;

        Ok(Self {
            bucket: require_bucket(bucket)?,
            checksum_crc32: header_str(parts, "x-amz-checksum-crc32"),
            checksum_crc32c: header_str(parts, "x-amz-checksum-crc32c"),
            checksum_crc64nvme: header_str(parts, "x-amz-checksum-crc64nvme"),
            checksum_sha1: header_str(parts, "x-amz-checksum-sha1"),
            checksum_sha256: header_str(parts, "x-amz-checksum-sha256"),
            checksum_type: header_enum(parts, "x-amz-checksum-type"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            if_match: header_str(parts, "If-Match"),
            if_none_match: header_str(parts, "If-None-Match"),
            key: require_key(key)?,
            mpu_object_size: header_parse(parts, "x-amz-mp-object-size"),
            multipart_upload: if body.is_empty() {
                None
            } else {
                Some(parse_xml_body::<CompletedMultipartUpload>(&body)?)
            },
            request_payer: header_enum(parts, "x-amz-request-payer"),
            sse_customer_algorithm: header_str(
                parts,
                "x-amz-server-side-encryption-customer-algorithm",
            ),
            sse_customer_key: header_str(parts, "x-amz-server-side-encryption-customer-key"),
            sse_customer_key_md5: header_str(
                parts,
                "x-amz-server-side-encryption-customer-key-MD5",
            ),
            upload_id,
        })
    }
}

impl FromS3Request for AbortMultipartUploadInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        let upload_id = query_param(query_params, "uploadId").ok_or_else(|| {
            S3Error::with_message(
                S3ErrorCode::InvalidRequest,
                "uploadId query parameter is required",
            )
        })?;

        Ok(Self {
            bucket: require_bucket(bucket)?,
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            if_match_initiated_time: header_timestamp(parts, "x-amz-if-match-initiated-time"),
            key: require_key(key)?,
            request_payer: header_enum(parts, "x-amz-request-payer"),
            upload_id,
        })
    }
}

impl FromS3Request for ListPartsInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        let upload_id = query_param(query_params, "uploadId").ok_or_else(|| {
            S3Error::with_message(
                S3ErrorCode::InvalidRequest,
                "uploadId query parameter is required",
            )
        })?;

        Ok(Self {
            bucket: require_bucket(bucket)?,
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            key: require_key(key)?,
            max_parts: query_param_parse(query_params, "max-parts"),
            part_number_marker: query_param(query_params, "part-number-marker"),
            request_payer: header_enum(parts, "x-amz-request-payer"),
            sse_customer_algorithm: header_str(
                parts,
                "x-amz-server-side-encryption-customer-algorithm",
            ),
            sse_customer_key: header_str(parts, "x-amz-server-side-encryption-customer-key"),
            sse_customer_key_md5: header_str(
                parts,
                "x-amz-server-side-encryption-customer-key-MD5",
            ),
            upload_id,
        })
    }
}

impl FromS3Request for ListMultipartUploadsInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        query_params: &[(String, String)],
        _body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            delimiter: query_param(query_params, "delimiter"),
            encoding_type: query_param(query_params, "encoding-type")
                .as_deref()
                .map(Into::into),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            key_marker: query_param(query_params, "key-marker"),
            max_uploads: query_param_parse(query_params, "max-uploads"),
            prefix: query_param(query_params, "prefix"),
            request_payer: header_enum(parts, "x-amz-request-payer"),
            upload_id_marker: query_param(query_params, "upload-id-marker"),
        })
    }
}

// --- Config PUT operations (bucket config with XML body) ---

impl FromS3Request for PutBucketAccelerateConfigurationInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            accelerate_configuration: parse_xml_body::<AccelerateConfiguration>(&body)?,
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
        })
    }
}

impl FromS3Request for PutBucketCorsInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            cors_configuration: parse_xml_body::<CORSConfiguration>(&body)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
        })
    }
}

impl FromS3Request for PutBucketEncryptionInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            server_side_encryption_configuration: parse_xml_body::<
                ServerSideEncryptionConfiguration,
            >(&body)?,
        })
    }
}

impl FromS3Request for PutBucketLifecycleConfigurationInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            lifecycle_configuration: if body.is_empty() {
                None
            } else {
                Some(parse_xml_body::<BucketLifecycleConfiguration>(&body)?)
            },
            transition_default_minimum_object_size: header_enum(
                parts,
                "x-amz-transition-default-minimum-object-size",
            ),
        })
    }
}

impl FromS3Request for PutBucketLoggingInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            bucket_logging_status: parse_xml_body::<BucketLoggingStatus>(&body)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
        })
    }
}

impl FromS3Request for PutBucketNotificationConfigurationInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            notification_configuration: parse_xml_body::<NotificationConfiguration>(&body)?,
            skip_destination_validation: header_bool(parts, "x-amz-skip-destination-validation"),
        })
    }
}

impl FromS3Request for PutBucketOwnershipControlsInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            ownership_controls: parse_xml_body::<OwnershipControls>(&body)?,
        })
    }
}

impl FromS3Request for PutBucketPolicyInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        let policy = String::from_utf8(body.to_vec()).map_err(|e| {
            S3Error::malformed_xml(format!("Failed to parse policy body as UTF-8: {e}"))
        })?;
        Ok(Self {
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            confirm_remove_self_bucket_access: header_bool(
                parts,
                "x-amz-confirm-remove-self-bucket-access",
            ),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            policy,
        })
    }
}

impl FromS3Request for PutBucketRequestPaymentInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            request_payment_configuration: parse_xml_body::<RequestPaymentConfiguration>(&body)?,
        })
    }
}

impl FromS3Request for PutBucketTaggingInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            tagging: parse_xml_body::<Tagging>(&body)?,
        })
    }
}

impl FromS3Request for PutBucketVersioningInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            mfa: header_str(parts, "x-amz-mfa"),
            versioning_configuration: parse_xml_body::<VersioningConfiguration>(&body)?,
        })
    }
}

impl FromS3Request for PutBucketWebsiteInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            website_configuration: parse_xml_body::<WebsiteConfiguration>(&body)?,
        })
    }
}

impl FromS3Request for PutPublicAccessBlockInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            public_access_block_configuration: parse_xml_body::<PublicAccessBlockConfiguration>(
                &body,
            )?,
        })
    }
}

// --- ACL PUT operations ---

impl FromS3Request for PutBucketAclInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            acl: header_enum(parts, "x-amz-acl"),
            access_control_policy: if body.is_empty() {
                None
            } else {
                Some(parse_xml_body::<AccessControlPolicy>(&body)?)
            },
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            grant_full_control: header_str(parts, "x-amz-grant-full-control"),
            grant_read: header_str(parts, "x-amz-grant-read"),
            grant_read_acp: header_str(parts, "x-amz-grant-read-acp"),
            grant_write: header_str(parts, "x-amz-grant-write"),
            grant_write_acp: header_str(parts, "x-amz-grant-write-acp"),
        })
    }
}

impl FromS3Request for PutObjectAclInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            acl: header_enum(parts, "x-amz-acl"),
            access_control_policy: if body.is_empty() {
                None
            } else {
                Some(parse_xml_body::<AccessControlPolicy>(&body)?)
            },
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            grant_full_control: header_str(parts, "x-amz-grant-full-control"),
            grant_read: header_str(parts, "x-amz-grant-read"),
            grant_read_acp: header_str(parts, "x-amz-grant-read-acp"),
            grant_write: header_str(parts, "x-amz-grant-write"),
            grant_write_acp: header_str(parts, "x-amz-grant-write-acp"),
            key: require_key(key)?,
            request_payer: header_enum(parts, "x-amz-request-payer"),
            version_id: query_param(query_params, "versionId"),
        })
    }
}

impl FromS3Request for PutObjectTaggingInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            key: require_key(key)?,
            request_payer: header_enum(parts, "x-amz-request-payer"),
            tagging: parse_xml_body::<Tagging>(&body)?,
            version_id: query_param(query_params, "versionId"),
        })
    }
}

impl FromS3Request for PutObjectLegalHoldInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            key: require_key(key)?,
            legal_hold: if body.is_empty() {
                None
            } else {
                Some(parse_xml_body::<ObjectLockLegalHold>(&body)?)
            },
            request_payer: header_enum(parts, "x-amz-request-payer"),
            version_id: query_param(query_params, "versionId"),
        })
    }
}

impl FromS3Request for PutObjectLockConfigurationInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        _key: Option<&str>,
        _query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            object_lock_configuration: if body.is_empty() {
                None
            } else {
                Some(parse_xml_body::<ObjectLockConfiguration>(&body)?)
            },
            request_payer: header_enum(parts, "x-amz-request-payer"),
            token: header_str(parts, "x-amz-bucket-object-lock-token"),
        })
    }
}

impl FromS3Request for PutObjectRetentionInput {
    fn from_s3_request(
        parts: &http::request::Parts,
        bucket: Option<&str>,
        key: Option<&str>,
        query_params: &[(String, String)],
        body: Bytes,
    ) -> Result<Self, S3Error> {
        Ok(Self {
            bucket: require_bucket(bucket)?,
            bypass_governance_retention: header_bool(parts, "x-amz-bypass-governance-retention"),
            checksum_algorithm: header_enum(parts, "x-amz-sdk-checksum-algorithm"),
            content_md5: header_str(parts, "Content-MD5"),
            expected_bucket_owner: header_str(parts, "x-amz-expected-bucket-owner"),
            key: require_key(key)?,
            request_payer: header_enum(parts, "x-amz-request-payer"),
            retention: if body.is_empty() {
                None
            } else {
                Some(parse_xml_body::<ObjectLockRetention>(&body)?)
            },
            version_id: query_param(query_params, "versionId"),
        })
    }
}

#[cfg(test)]
mod tests {
    use http::Request;

    use super::*;

    #[test]
    fn test_should_extract_list_buckets_input() {
        let req = Request::builder()
            .method(http::Method::GET)
            .uri("/?prefix=test&max-buckets=10")
            .body(())
            .expect("valid request");
        let (parts, ()) = req.into_parts();
        let params = vec![
            ("prefix".to_owned(), "test".to_owned()),
            ("max-buckets".to_owned(), "10".to_owned()),
        ];

        let input = ListBucketsInput::from_s3_request(&parts, None, None, &params, Bytes::new())
            .expect("should parse");
        assert_eq!(input.prefix.as_deref(), Some("test"));
        assert_eq!(input.max_buckets, Some(10));
    }

    #[test]
    fn test_should_extract_put_object_input() {
        let body = Bytes::from("hello world");
        let req = Request::builder()
            .method(http::Method::PUT)
            .uri("/mybucket/mykey")
            .header("Content-Type", "text/plain")
            .header("Content-Length", "11")
            .header("x-amz-meta-author", "test")
            .header("x-amz-storage-class", "STANDARD")
            .body(())
            .expect("valid request");
        let (parts, ()) = req.into_parts();

        let input =
            PutObjectInput::from_s3_request(&parts, Some("mybucket"), Some("mykey"), &[], body)
                .expect("should parse");

        assert_eq!(input.bucket, "mybucket");
        assert_eq!(input.key, "mykey");
        assert_eq!(input.content_type.as_deref(), Some("text/plain"));
        assert_eq!(input.content_length, Some(11));
        assert_eq!(input.metadata.get("author"), Some(&"test".to_owned()));
        assert!(input.body.is_some());
    }

    #[test]
    fn test_should_extract_get_object_input() {
        let req = Request::builder()
            .method(http::Method::GET)
            .uri("/mybucket/mykey?versionId=v1")
            .header("Range", "bytes=0-100")
            .header("If-Match", "\"etag123\"")
            .body(())
            .expect("valid request");
        let (parts, ()) = req.into_parts();
        let params = vec![("versionId".to_owned(), "v1".to_owned())];

        let input = GetObjectInput::from_s3_request(
            &parts,
            Some("mybucket"),
            Some("mykey"),
            &params,
            Bytes::new(),
        )
        .expect("should parse");

        assert_eq!(input.bucket, "mybucket");
        assert_eq!(input.key, "mykey");
        assert_eq!(input.version_id.as_deref(), Some("v1"));
        assert_eq!(input.range.as_deref(), Some("bytes=0-100"));
        assert_eq!(input.if_match.as_deref(), Some("\"etag123\""));
    }

    #[test]
    fn test_should_collect_metadata_headers() {
        let req = Request::builder()
            .method(http::Method::PUT)
            .uri("/mybucket/mykey")
            .header("x-amz-meta-foo", "bar")
            .header("x-amz-meta-baz", "qux")
            .header("x-amz-not-meta", "ignored")
            .body(())
            .expect("valid request");
        let (parts, ()) = req.into_parts();

        let metadata = collect_metadata(&parts);
        assert_eq!(metadata.len(), 2);
        assert_eq!(metadata.get("foo"), Some(&"bar".to_owned()));
        assert_eq!(metadata.get("baz"), Some(&"qux".to_owned()));
    }

    #[test]
    fn test_should_require_bucket_when_needed() {
        let req = Request::builder()
            .method(http::Method::DELETE)
            .uri("/")
            .body(())
            .expect("valid request");
        let (parts, ()) = req.into_parts();

        let err =
            DeleteBucketInput::from_s3_request(&parts, None, None, &[], Bytes::new()).unwrap_err();
        assert_eq!(err.code, S3ErrorCode::InvalidRequest);
    }

    #[test]
    fn test_should_require_key_when_needed() {
        let req = Request::builder()
            .method(http::Method::GET)
            .uri("/mybucket")
            .body(())
            .expect("valid request");
        let (parts, ()) = req.into_parts();

        let err =
            GetObjectInput::from_s3_request(&parts, Some("mybucket"), None, &[], Bytes::new())
                .unwrap_err();
        assert_eq!(err.code, S3ErrorCode::InvalidRequest);
    }

    #[test]
    fn test_should_parse_upload_part_input() {
        let body = Bytes::from("part data");
        let req = Request::builder()
            .method(http::Method::PUT)
            .uri("/mybucket/mykey?partNumber=3&uploadId=upload123")
            .header("Content-Length", "9")
            .body(())
            .expect("valid request");
        let (parts, ()) = req.into_parts();
        let params = vec![
            ("partNumber".to_owned(), "3".to_owned()),
            ("uploadId".to_owned(), "upload123".to_owned()),
        ];

        let input = UploadPartInput::from_s3_request(
            &parts,
            Some("mybucket"),
            Some("mykey"),
            &params,
            body,
        )
        .expect("should parse");

        assert_eq!(input.part_number, 3);
        assert_eq!(input.upload_id, "upload123");
        assert_eq!(input.content_length, Some(9));
        assert!(input.body.is_some());
    }

    #[test]
    fn test_should_parse_http_date_rfc3339() {
        let dt = parse_http_date("2024-01-15T10:30:00Z");
        assert!(dt.is_some());
    }

    #[test]
    fn test_should_parse_http_date_rfc2822() {
        let dt = parse_http_date("Mon, 15 Jan 2024 10:30:00 +0000");
        assert!(dt.is_some());
    }
}
