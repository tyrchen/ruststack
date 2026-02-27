//! Auto-generated from AWS S3 Smithy model. DO NOT EDIT.

use crate::types::{EncodingType, OptionalObjectAttributes, RequestPayer};

/// S3 ListObjectVersionsInput.
#[derive(Debug, Clone, Default)]
pub struct ListObjectVersionsInput {
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
    /// HTTP query: `max-keys`.
    pub max_keys: Option<i32>,
    /// HTTP header: `x-amz-optional-object-attributes`.
    pub optional_object_attributes: Vec<OptionalObjectAttributes>,
    /// HTTP query: `prefix`.
    pub prefix: Option<String>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP query: `version-id-marker`.
    pub version_id_marker: Option<String>,
}

/// S3 ListObjectsInput.
#[derive(Debug, Clone, Default)]
pub struct ListObjectsInput {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP query: `delimiter`.
    pub delimiter: Option<String>,
    /// HTTP query: `encoding-type`.
    pub encoding_type: Option<EncodingType>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP query: `marker`.
    pub marker: Option<String>,
    /// HTTP query: `max-keys`.
    pub max_keys: Option<i32>,
    /// HTTP header: `x-amz-optional-object-attributes`.
    pub optional_object_attributes: Vec<OptionalObjectAttributes>,
    /// HTTP query: `prefix`.
    pub prefix: Option<String>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
}

/// S3 ListObjectsV2Input.
#[derive(Debug, Clone, Default)]
pub struct ListObjectsV2Input {
    /// HTTP label (URI path).
    pub bucket: String,
    /// HTTP query: `continuation-token`.
    pub continuation_token: Option<String>,
    /// HTTP query: `delimiter`.
    pub delimiter: Option<String>,
    /// HTTP query: `encoding-type`.
    pub encoding_type: Option<EncodingType>,
    /// HTTP header: `x-amz-expected-bucket-owner`.
    pub expected_bucket_owner: Option<String>,
    /// HTTP query: `fetch-owner`.
    pub fetch_owner: Option<bool>,
    /// HTTP query: `max-keys`.
    pub max_keys: Option<i32>,
    /// HTTP header: `x-amz-optional-object-attributes`.
    pub optional_object_attributes: Vec<OptionalObjectAttributes>,
    /// HTTP query: `prefix`.
    pub prefix: Option<String>,
    /// HTTP header: `x-amz-request-payer`.
    pub request_payer: Option<RequestPayer>,
    /// HTTP query: `start-after`.
    pub start_after: Option<String>,
}
