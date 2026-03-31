//! Auto-generated from AWS S3 Smithy model. DO NOT EDIT.

use crate::types::{
    CommonPrefix, DeleteMarkerEntry, EncodingType, Object, ObjectVersion, RequestCharged,
};

/// S3 ListObjectVersionsOutput.
#[derive(Debug, Clone, Default)]
pub struct ListObjectVersionsOutput {
    pub common_prefixes: Vec<CommonPrefix>,
    pub delete_markers: Vec<DeleteMarkerEntry>,
    pub delimiter: Option<String>,
    pub encoding_type: Option<EncodingType>,
    pub is_truncated: Option<bool>,
    pub key_marker: Option<String>,
    pub max_keys: Option<i32>,
    pub name: Option<String>,
    pub next_key_marker: Option<String>,
    pub next_version_id_marker: Option<String>,
    pub prefix: Option<String>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
    pub version_id_marker: Option<String>,
    pub versions: Vec<ObjectVersion>,
}

/// S3 ListObjectsOutput.
#[derive(Debug, Clone, Default)]
pub struct ListObjectsOutput {
    pub common_prefixes: Vec<CommonPrefix>,
    pub contents: Vec<Object>,
    pub delimiter: Option<String>,
    pub encoding_type: Option<EncodingType>,
    pub is_truncated: Option<bool>,
    pub marker: Option<String>,
    pub max_keys: Option<i32>,
    pub name: Option<String>,
    pub next_marker: Option<String>,
    pub prefix: Option<String>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
}

/// S3 ListObjectsV2Output.
#[derive(Debug, Clone, Default)]
pub struct ListObjectsV2Output {
    pub common_prefixes: Vec<CommonPrefix>,
    pub contents: Vec<Object>,
    pub continuation_token: Option<String>,
    pub delimiter: Option<String>,
    pub encoding_type: Option<EncodingType>,
    pub is_truncated: Option<bool>,
    pub key_count: Option<i32>,
    pub max_keys: Option<i32>,
    pub name: Option<String>,
    pub next_continuation_token: Option<String>,
    pub prefix: Option<String>,
    /// HTTP header: `x-amz-request-charged`.
    pub request_charged: Option<RequestCharged>,
    pub start_after: Option<String>,
}
