//! List operation handlers.
//!
//! Implements `list_objects` (v1), `list_objects_v2`, and `list_object_versions`.

// The s3s DTO module contains dozens of types we reference; wildcard is clearer.
#[allow(clippy::wildcard_imports)]
use s3s::dto::*;
use s3s::{S3Request, S3Response, S3Result};
use tracing::debug;

use crate::error::S3ServiceError;
use crate::provider::RustStackS3;
use crate::state::keystore::VersionListEntry;
use crate::state::object::Owner as InternalOwner;
use crate::utils::{decode_continuation_token, encode_continuation_token};

use super::bucket::chrono_to_timestamp;

/// Default maximum number of keys returned in a single listing response.
const DEFAULT_MAX_KEYS: i32 = 1000;

/// Convert an internal [`crate::state::object::S3Object`] to an s3s [`Object`] DTO.
#[allow(clippy::cast_possible_wrap)]
fn to_s3_object(obj: &crate::state::object::S3Object) -> Object {
    let owner = Owner {
        display_name: Some(obj.owner.display_name.clone()),
        id: Some(obj.owner.id.clone()),
    };
    Object {
        checksum_algorithm: None,
        checksum_type: None,
        e_tag: Some(obj.etag.clone()),
        key: Some(obj.key.clone()),
        last_modified: Some(chrono_to_timestamp(obj.last_modified)),
        owner: Some(owner),
        restore_status: None,
        size: Some(obj.size as i64),
        storage_class: Some(ObjectStorageClass::from(obj.storage_class.clone())),
    }
}

/// Convert an internal [`InternalOwner`] to an s3s [`Owner`] DTO.
fn to_s3_owner(owner: &InternalOwner) -> Owner {
    Owner {
        display_name: Some(owner.display_name.clone()),
        id: Some(owner.id.clone()),
    }
}

/// Convert common prefix strings to s3s [`CommonPrefix`] DTOs.
fn to_common_prefixes(prefixes: &[String]) -> Option<CommonPrefixList> {
    if prefixes.is_empty() {
        None
    } else {
        let list: Vec<CommonPrefix> = prefixes
            .iter()
            .map(|p| CommonPrefix {
                prefix: Some(p.clone()),
            })
            .collect();
        Some(list)
    }
}

// AWS S3 DTOs use signed integers (i32/i64) for inherently non-negative values.
// These handler methods must remain async to match the s3s::S3 trait interface.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unused_async
)]
impl RustStackS3 {
    /// List objects (v1 API).
    pub(crate) async fn handle_list_objects(
        &self,
        req: S3Request<ListObjectsInput>,
    ) -> S3Result<S3Response<ListObjectsOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let prefix = req.input.prefix.as_deref().unwrap_or("");
        let delimiter = req.input.delimiter.as_deref().unwrap_or("");
        let marker = req.input.marker.as_deref().unwrap_or("");
        let max_keys = req.input.max_keys.unwrap_or(DEFAULT_MAX_KEYS);
        let max_keys_usize = usize::try_from(max_keys).unwrap_or(1000);

        let store = bucket.objects.read();
        let result = store.list_objects(prefix, delimiter, marker, max_keys_usize);
        drop(store);
        drop(bucket);

        let contents: Vec<Object> = result.objects.iter().map(to_s3_object).collect();
        let common_prefixes = to_common_prefixes(&result.common_prefixes);

        let next_marker = if result.is_truncated {
            result.next_marker.clone()
        } else {
            None
        };

        debug!(
            bucket = %bucket_name,
            prefix = %prefix,
            count = contents.len(),
            is_truncated = result.is_truncated,
            "list_objects completed"
        );

        let output = ListObjectsOutput {
            common_prefixes,
            contents: if contents.is_empty() {
                None
            } else {
                Some(contents)
            },
            delimiter: req.input.delimiter,
            encoding_type: req.input.encoding_type,
            is_truncated: Some(result.is_truncated),
            marker: req.input.marker,
            max_keys: Some(max_keys),
            name: Some(bucket_name),
            next_marker,
            prefix: req.input.prefix,
            request_charged: None,
        };
        Ok(S3Response::new(output))
    }

    /// List objects (v2 API with continuation tokens).
    pub(crate) async fn handle_list_objects_v2(
        &self,
        req: S3Request<ListObjectsV2Input>,
    ) -> S3Result<S3Response<ListObjectsV2Output>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let prefix = req.input.prefix.as_deref().unwrap_or("");
        let delimiter = req.input.delimiter.as_deref().unwrap_or("");
        let max_keys = req.input.max_keys.unwrap_or(DEFAULT_MAX_KEYS);
        let max_keys_usize = usize::try_from(max_keys).unwrap_or(1000);
        let fetch_owner = req.input.fetch_owner.unwrap_or(false);

        // Determine start_after: either from continuation token or start_after param.
        let decoded_token = if let Some(ref token) = req.input.continuation_token {
            Some(decode_continuation_token(token).map_err(S3ServiceError::into_s3_error)?)
        } else {
            None
        };
        let start_after = decoded_token
            .as_deref()
            .or(req.input.start_after.as_deref())
            .unwrap_or("");

        let store = bucket.objects.read();
        let result = store.list_objects(prefix, delimiter, start_after, max_keys_usize);
        drop(store);
        drop(bucket);

        let contents: Vec<Object> = result
            .objects
            .iter()
            .map(|obj| {
                let mut s3_obj = to_s3_object(obj);
                if !fetch_owner {
                    s3_obj.owner = None;
                }
                s3_obj
            })
            .collect();
        let common_prefixes = to_common_prefixes(&result.common_prefixes);

        let next_continuation_token = if result.is_truncated {
            result
                .next_marker
                .as_ref()
                .map(|m| encode_continuation_token(m))
        } else {
            None
        };

        let key_count = contents.len() as i32;

        debug!(
            bucket = %bucket_name,
            prefix = %prefix,
            count = key_count,
            is_truncated = result.is_truncated,
            "list_objects_v2 completed"
        );

        let output = ListObjectsV2Output {
            common_prefixes,
            contents: if contents.is_empty() {
                None
            } else {
                Some(contents)
            },
            continuation_token: req.input.continuation_token,
            delimiter: req.input.delimiter,
            encoding_type: req.input.encoding_type,
            is_truncated: Some(result.is_truncated),
            key_count: Some(key_count),
            max_keys: Some(max_keys),
            name: Some(bucket_name),
            next_continuation_token,
            prefix: req.input.prefix,
            request_charged: None,
            start_after: req.input.start_after,
        };
        Ok(S3Response::new(output))
    }

    /// List object versions.
    pub(crate) async fn handle_list_object_versions(
        &self,
        req: S3Request<ListObjectVersionsInput>,
    ) -> S3Result<S3Response<ListObjectVersionsOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let prefix = req.input.prefix.as_deref().unwrap_or("");
        let delimiter = req.input.delimiter.as_deref().unwrap_or("");
        let key_marker = req.input.key_marker.as_deref().unwrap_or("");
        let version_id_marker = req.input.version_id_marker.as_deref().unwrap_or("");
        let max_keys = req.input.max_keys.unwrap_or(DEFAULT_MAX_KEYS);
        let max_keys_usize = usize::try_from(max_keys).unwrap_or(1000);

        let store = bucket.objects.read();
        let result = store.list_object_versions(
            prefix,
            delimiter,
            key_marker,
            version_id_marker,
            max_keys_usize,
        );
        drop(store);
        drop(bucket);

        // Separate versions and delete markers.
        let (version_entries, delete_marker_entries) =
            partition_version_list_entries(&result.versions);

        let common_prefixes = to_common_prefixes(&result.common_prefixes);

        debug!(
            bucket = %bucket_name,
            prefix = %prefix,
            versions = version_entries.len(),
            delete_markers = delete_marker_entries.len(),
            is_truncated = result.is_truncated,
            "list_object_versions completed"
        );

        let output = ListObjectVersionsOutput {
            common_prefixes,
            delete_markers: if delete_marker_entries.is_empty() {
                None
            } else {
                Some(delete_marker_entries)
            },
            delimiter: req.input.delimiter,
            encoding_type: req.input.encoding_type,
            is_truncated: Some(result.is_truncated),
            key_marker: req.input.key_marker,
            max_keys: Some(max_keys),
            name: Some(bucket_name),
            next_key_marker: result.next_key_marker,
            next_version_id_marker: result.next_version_id_marker,
            prefix: req.input.prefix,
            request_charged: None,
            version_id_marker: req.input.version_id_marker,
            versions: if version_entries.is_empty() {
                None
            } else {
                Some(version_entries)
            },
        };
        Ok(S3Response::new(output))
    }
}

/// Partition a list of [`VersionListEntry`] into s3s [`ObjectVersion`] and
/// [`DeleteMarkerEntry`] DTOs.
#[allow(clippy::cast_possible_wrap)]
fn partition_version_list_entries(
    entries: &[VersionListEntry],
) -> (Vec<s3s::dto::ObjectVersion>, Vec<DeleteMarkerEntry>) {
    let mut versions = Vec::new();
    let mut delete_markers = Vec::new();

    for entry in entries {
        match &entry.version {
            crate::state::object::ObjectVersion::Object(obj) => {
                let owner = to_s3_owner(&obj.owner);
                versions.push(s3s::dto::ObjectVersion {
                    checksum_algorithm: None,
                    checksum_type: None,
                    e_tag: Some(obj.etag.clone()),
                    is_latest: Some(entry.is_latest),
                    key: Some(obj.key.clone()),
                    last_modified: Some(chrono_to_timestamp(obj.last_modified)),
                    owner: Some(owner),
                    restore_status: None,
                    size: Some(obj.size as i64),
                    storage_class: Some(ObjectVersionStorageClass::from(obj.storage_class.clone())),
                    version_id: Some(obj.version_id.clone()),
                });
            }
            crate::state::object::ObjectVersion::DeleteMarker(dm) => {
                let owner = to_s3_owner(&dm.owner);
                delete_markers.push(DeleteMarkerEntry {
                    is_latest: Some(entry.is_latest),
                    key: Some(dm.key.clone()),
                    last_modified: Some(chrono_to_timestamp(dm.last_modified)),
                    owner: Some(owner),
                    version_id: Some(dm.version_id.clone()),
                });
            }
        }
    }

    (versions, delete_markers)
}
