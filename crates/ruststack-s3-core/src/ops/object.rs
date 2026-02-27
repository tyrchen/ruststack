//! Object CRUD operation handlers.
//!
//! Implements `put_object`, `get_object`, `head_object`, `delete_object`,
//! `delete_objects`, and `copy_object`.

use bytes::{Bytes, BytesMut};
use chrono::Utc;
use futures::TryStreamExt;
// The s3s DTO module contains dozens of types we reference; wildcard is clearer.
#[allow(clippy::wildcard_imports)]
use s3s::dto::*;
use s3s::{S3Request, S3Response, S3Result};
use tracing::debug;

use crate::error::S3ServiceError;
use crate::provider::RustStackS3;
use crate::state::object::{
    CannedAcl, ChecksumData, ObjectMetadata, Owner as InternalOwner, S3Object,
};
use crate::utils::{is_valid_if_match, is_valid_if_none_match};
use crate::validation::{validate_metadata, validate_object_key};

use super::bucket::chrono_to_timestamp;

// AWS S3 DTOs use signed integers (i32/i64) for inherently non-negative values
// (sizes, part counts). Casting from u64/u32/usize is safe in practice.
// These handler methods must remain async to match the s3s::S3 trait interface.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unused_async
)]
impl RustStackS3 {
    /// Put (upload) a new object.
    pub(crate) async fn handle_put_object(
        &self,
        mut req: S3Request<PutObjectInput>,
    ) -> S3Result<S3Response<PutObjectOutput>> {
        let bucket_name = req.input.bucket.clone();
        let key = req.input.key.clone();

        validate_object_key(&key).map_err(S3ServiceError::into_s3_error)?;

        // Verify bucket exists.
        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        // Take the body out before borrowing other fields from input.
        let body = req.input.body.take();

        // Collect the body.
        let body_data = collect_body(body).await?;

        // Extract metadata from the request.
        let metadata = build_metadata(&req.input, &req.headers);
        validate_metadata(&metadata.user_metadata).map_err(S3ServiceError::into_s3_error)?;

        // Determine version ID based on versioning status.
        let version_id = if bucket.is_versioning_enabled() {
            crate::utils::generate_version_id()
        } else {
            "null".to_owned()
        };

        // Write to storage.
        let write_result = self
            .storage
            .write_object(&bucket_name, &key, &version_id, body_data.clone())
            .await
            .map_err(|e| S3ServiceError::Internal(anyhow::anyhow!("{e}")).into_s3_error())?;

        // Extract checksum from the request, if provided.
        let checksum = extract_checksum_from_put(&req.input);

        // Build the S3Object.
        let owner = InternalOwner::default();
        let obj = S3Object {
            key: key.clone(),
            version_id: version_id.clone(),
            etag: write_result.etag.clone(),
            size: write_result.size,
            last_modified: Utc::now(),
            storage_class: req
                .input
                .storage_class
                .as_ref()
                .map_or_else(|| "STANDARD".to_owned(), |s| s.as_str().to_owned()),
            metadata,
            owner,
            checksum,
            parts_count: None,
            part_etags: Vec::new(),
        };

        // Store the object metadata.
        {
            let mut store = bucket.objects.write();
            store.put(obj);
        }

        debug!(bucket = %bucket_name, key = %key, version_id = %version_id, "put_object completed");

        let real_version_id = if version_id == "null" {
            None
        } else {
            Some(version_id)
        };

        let output = PutObjectOutput {
            bucket_key_enabled: None,
            checksum_crc32: None,
            checksum_crc32c: None,
            checksum_crc64nvme: None,
            checksum_sha1: None,
            checksum_sha256: None,
            checksum_type: None,
            e_tag: Some(write_result.etag),
            expiration: None,
            request_charged: None,
            sse_customer_algorithm: None,
            sse_customer_key_md5: None,
            ssekms_encryption_context: None,
            ssekms_key_id: None,
            server_side_encryption: None,
            size: None,
            version_id: real_version_id,
        };
        Ok(S3Response::new(output))
    }

    /// Get (download) an object.
    #[allow(clippy::too_many_lines)]
    pub(crate) async fn handle_get_object(
        &self,
        req: S3Request<GetObjectInput>,
    ) -> S3Result<S3Response<GetObjectOutput>> {
        let bucket_name = req.input.bucket;
        let key = req.input.key;

        // Look up the object and extract all needed data while holding the lock.
        // The lock must be dropped before any `.await` calls since parking_lot
        // guards are `!Send`.
        let (
            obj_size,
            obj_etag,
            obj_last_modified,
            obj_version_id,
            obj_storage_class,
            obj_meta,
            obj_parts_count,
            version_for_storage,
        ) = {
            let bucket = self
                .state
                .get_bucket(&bucket_name)
                .map_err(S3ServiceError::into_s3_error)?;

            let store = bucket.objects.read();
            let obj = if let Some(version_id) = &req.input.version_id {
                store.get_version(&key, version_id).ok_or_else(|| {
                    S3ServiceError::NoSuchVersion {
                        key: key.clone(),
                        version_id: version_id.clone(),
                    }
                    .into_s3_error()
                })?
            } else {
                store
                    .get(&key)
                    .ok_or_else(|| S3ServiceError::NoSuchKey { key: key.clone() }.into_s3_error())?
            };

            // Conditional request checks.
            if let Some(ref if_match) = req.input.if_match {
                if !is_valid_if_match(&obj.etag, if_match) {
                    return Err(s3s::s3_error!(PreconditionFailed));
                }
            }
            if let Some(ref if_none_match) = req.input.if_none_match {
                if !is_valid_if_none_match(&obj.etag, if_none_match) {
                    return Err(s3s::s3_error!(NotModified));
                }
            }

            let version_id_opt = if obj.version_id == "null" {
                None
            } else {
                Some(obj.version_id.clone())
            };

            (
                obj.size,
                obj.etag.clone(),
                obj.last_modified,
                version_id_opt,
                obj.storage_class.clone(),
                obj.metadata.clone(),
                obj.parts_count,
                obj.version_id.clone(),
            )
        };

        // Parse range header if provided.
        let range = if let Some(ref range_value) = req.input.range {
            let std_range = range_value
                .check(obj_size)
                .map_err(|_| S3ServiceError::InvalidRange.into_s3_error())?;
            Some((std_range.start, std_range.end - 1))
        } else {
            None
        };

        // Read data from storage.
        let data = self
            .storage
            .read_object(&bucket_name, &key, &version_for_storage, range)
            .await
            .map_err(|e| S3ServiceError::Internal(anyhow::anyhow!("{e}")).into_s3_error())?;

        let content_length = data.len() as i64;

        // Build the streaming body from the data bytes.
        let body = StreamingBlob::wrap(futures::stream::once(async move {
            Ok::<_, std::io::Error>(data)
        }));

        let content_range = if let Some((start, end)) = range {
            Some(format!("bytes {start}-{end}/{obj_size}"))
        } else {
            None
        };

        let output = GetObjectOutput {
            accept_ranges: Some("bytes".to_owned()),
            body: Some(body),
            bucket_key_enabled: None,
            cache_control: obj_meta.cache_control,
            checksum_crc32: None,
            checksum_crc32c: None,
            checksum_crc64nvme: None,
            checksum_sha1: None,
            checksum_sha256: None,
            checksum_type: None,
            content_disposition: obj_meta.content_disposition,
            content_encoding: obj_meta.content_encoding,
            content_language: obj_meta.content_language,
            content_length: Some(content_length),
            content_range,
            content_type: Some(
                obj_meta
                    .content_type
                    .as_deref()
                    .unwrap_or("application/octet-stream")
                    .parse::<mime::Mime>()
                    .unwrap_or(mime::APPLICATION_OCTET_STREAM),
            ),
            delete_marker: None,
            e_tag: Some(obj_etag),
            expiration: None,
            expires: None,
            last_modified: Some(chrono_to_timestamp(obj_last_modified)),
            metadata: if obj_meta.user_metadata.is_empty() {
                None
            } else {
                Some(obj_meta.user_metadata)
            },
            missing_meta: None,
            object_lock_legal_hold_status: obj_meta
                .object_lock_legal_hold
                .filter(|&v| v)
                .map(|_| ObjectLockLegalHoldStatus::from_static("ON")),
            object_lock_mode: obj_meta.object_lock_mode.map(ObjectLockMode::from),
            object_lock_retain_until_date: obj_meta
                .object_lock_retain_until
                .map(chrono_to_timestamp),
            parts_count: obj_parts_count.map(|n| n as i32),
            replication_status: None,
            request_charged: None,
            restore: None,
            sse_customer_algorithm: obj_meta.sse_customer_algorithm,
            sse_customer_key_md5: obj_meta.sse_customer_key_md5,
            ssekms_key_id: obj_meta.sse_kms_key_id,
            server_side_encryption: obj_meta.sse_algorithm.map(ServerSideEncryption::from),
            storage_class: Some(StorageClass::from(obj_storage_class)),
            tag_count: None,
            version_id: obj_version_id,
            website_redirect_location: None,
        };
        Ok(S3Response::new(output))
    }

    /// Head object (get metadata without body).
    pub(crate) async fn handle_head_object(
        &self,
        req: S3Request<HeadObjectInput>,
    ) -> S3Result<S3Response<HeadObjectOutput>> {
        let bucket_name = req.input.bucket;
        let key = req.input.key;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let store = bucket.objects.read();
        let obj = if let Some(version_id) = &req.input.version_id {
            store.get_version(&key, version_id).ok_or_else(|| {
                S3ServiceError::NoSuchVersion {
                    key: key.clone(),
                    version_id: version_id.clone(),
                }
                .into_s3_error()
            })?
        } else {
            store
                .get(&key)
                .ok_or_else(|| S3ServiceError::NoSuchKey { key: key.clone() }.into_s3_error())?
        };

        let obj_version_id = if obj.version_id == "null" {
            None
        } else {
            Some(obj.version_id.clone())
        };

        let output = HeadObjectOutput {
            accept_ranges: Some("bytes".to_owned()),
            archive_status: None,
            bucket_key_enabled: None,
            cache_control: obj.metadata.cache_control.clone(),
            checksum_crc32: None,
            checksum_crc32c: None,
            checksum_crc64nvme: None,
            checksum_sha1: None,
            checksum_sha256: None,
            checksum_type: None,
            content_disposition: obj.metadata.content_disposition.clone(),
            content_encoding: obj.metadata.content_encoding.clone(),
            content_language: obj.metadata.content_language.clone(),
            content_length: Some(obj.size as i64),
            content_range: None,
            content_type: Some(
                obj.metadata
                    .content_type
                    .as_deref()
                    .unwrap_or("application/octet-stream")
                    .parse::<mime::Mime>()
                    .unwrap_or(mime::APPLICATION_OCTET_STREAM),
            ),
            delete_marker: None,
            e_tag: Some(obj.etag.clone()),
            expiration: None,
            expires: None,
            last_modified: Some(chrono_to_timestamp(obj.last_modified)),
            metadata: if obj.metadata.user_metadata.is_empty() {
                None
            } else {
                Some(obj.metadata.user_metadata.clone())
            },
            missing_meta: None,
            object_lock_legal_hold_status: obj
                .metadata
                .object_lock_legal_hold
                .filter(|&v| v)
                .map(|_| ObjectLockLegalHoldStatus::from_static("ON")),
            object_lock_mode: obj
                .metadata
                .object_lock_mode
                .clone()
                .map(ObjectLockMode::from),
            object_lock_retain_until_date: obj
                .metadata
                .object_lock_retain_until
                .map(chrono_to_timestamp),
            parts_count: obj.parts_count.map(|n| n as i32),
            replication_status: None,
            request_charged: None,
            restore: None,
            sse_customer_algorithm: obj.metadata.sse_customer_algorithm.clone(),
            sse_customer_key_md5: obj.metadata.sse_customer_key_md5.clone(),
            ssekms_key_id: obj.metadata.sse_kms_key_id.clone(),
            server_side_encryption: obj
                .metadata
                .sse_algorithm
                .clone()
                .map(ServerSideEncryption::from),
            storage_class: Some(StorageClass::from(obj.storage_class.clone())),
            version_id: obj_version_id,
            website_redirect_location: None,
        };
        Ok(S3Response::new(output))
    }

    /// Delete a single object.
    pub(crate) async fn handle_delete_object(
        &self,
        req: S3Request<DeleteObjectInput>,
    ) -> S3Result<S3Response<DeleteObjectOutput>> {
        let bucket_name = req.input.bucket;
        let key = req.input.key;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let (delete_marker_version_id, version_id_to_remove) =
            if let Some(version_id) = &req.input.version_id {
                // Delete a specific version.
                let mut store = bucket.objects.write();
                let removed = store.delete_version(&key, version_id);
                if let Some(ref version) = removed {
                    self.storage
                        .delete_object(&bucket_name, &key, version.version_id());
                }
                let is_dm = removed
                    .as_ref()
                    .is_some_and(crate::state::object::ObjectVersion::is_delete_marker);
                (is_dm, removed.map(|v| v.version_id().to_owned()))
            } else {
                // Delete without version: in versioned bucket, create delete marker.
                let mut store = bucket.objects.write();
                let (dm_id, _had) = store.delete_versioned(&key, &InternalOwner::default());
                if dm_id.is_none() {
                    // Un-versioned bucket: remove the storage data.
                    self.storage.delete_object(&bucket_name, &key, "null");
                }
                (dm_id.is_some(), dm_id)
            };

        debug!(bucket = %bucket_name, key = %key, "delete_object completed");

        let output = DeleteObjectOutput {
            delete_marker: if delete_marker_version_id {
                Some(true)
            } else {
                None
            },
            request_charged: None,
            version_id: version_id_to_remove,
        };
        Ok(S3Response::new(output))
    }

    /// Delete multiple objects (bulk delete).
    pub(crate) async fn handle_delete_objects(
        &self,
        req: S3Request<DeleteObjectsInput>,
    ) -> S3Result<S3Response<DeleteObjectsOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let delete_request = req.input.delete;

        let objects = delete_request.objects;
        let quiet = delete_request.quiet.unwrap_or(false);

        let mut deleted: Vec<DeletedObject> = Vec::with_capacity(objects.len());
        let errors: Vec<Error> = Vec::new();

        for obj_id in objects {
            let key = obj_id.key;
            let version_id = obj_id.version_id;

            if let Some(vid) = &version_id {
                // Delete a specific version.
                let mut store = bucket.objects.write();
                let removed = store.delete_version(&key, vid);
                if let Some(ref version) = removed {
                    self.storage
                        .delete_object(&bucket_name, &key, version.version_id());
                }
                let is_dm = removed
                    .as_ref()
                    .is_some_and(crate::state::object::ObjectVersion::is_delete_marker);
                deleted.push(DeletedObject {
                    delete_marker: if is_dm { Some(true) } else { None },
                    delete_marker_version_id: if is_dm { Some(vid.clone()) } else { None },
                    key: Some(key),
                    version_id: Some(vid.clone()),
                });
            } else {
                // Delete without version.
                let mut store = bucket.objects.write();
                let (dm_id, _had) = store.delete_versioned(&key, &InternalOwner::default());
                if dm_id.is_none() {
                    self.storage.delete_object(&bucket_name, &key, "null");
                }
                deleted.push(DeletedObject {
                    delete_marker: dm_id.as_ref().map(|_| true),
                    delete_marker_version_id: dm_id.clone(),
                    key: Some(key),
                    version_id: dm_id,
                });
            }
        }

        debug!(
            bucket = %bucket_name,
            deleted_count = deleted.len(),
            error_count = errors.len(),
            "delete_objects completed"
        );

        let output = DeleteObjectsOutput {
            deleted: if quiet { None } else { Some(deleted) },
            errors: if errors.is_empty() {
                None
            } else {
                Some(errors)
            },
            request_charged: None,
        };
        Ok(S3Response::new(output))
    }

    /// Copy an object from a source to a destination.
    #[allow(clippy::too_many_lines)]
    pub(crate) async fn handle_copy_object(
        &self,
        req: S3Request<CopyObjectInput>,
    ) -> S3Result<S3Response<CopyObjectOutput>> {
        let dst_bucket = req.input.bucket.clone();
        let dst_key = req.input.key.clone();

        validate_object_key(&dst_key).map_err(S3ServiceError::into_s3_error)?;

        let (src_bucket, src_key, src_version_id) = match &req.input.copy_source {
            CopySource::Bucket {
                bucket,
                key,
                version_id,
            } => (
                bucket.to_string(),
                key.to_string(),
                version_id.as_ref().map(std::string::ToString::to_string),
            ),
            CopySource::AccessPoint { .. } => {
                return Err(s3s::s3_error!(
                    NotImplemented,
                    "AccessPoint copy source is not supported"
                ));
            }
        };

        // Look up source object to get its metadata.
        // Keep this entire block synchronous -- no awaits while the lock is held.
        let (src_metadata, src_version_for_storage) = {
            let src_bucket_ref = self
                .state
                .get_bucket(&src_bucket)
                .map_err(S3ServiceError::into_s3_error)?;

            let src_store = src_bucket_ref.objects.read();
            let src_obj = if let Some(ref vid) = src_version_id {
                src_store.get_version(&src_key, vid).ok_or_else(|| {
                    S3ServiceError::NoSuchVersion {
                        key: src_key.clone(),
                        version_id: vid.clone(),
                    }
                    .into_s3_error()
                })?
            } else {
                src_store.get(&src_key).ok_or_else(|| {
                    S3ServiceError::NoSuchKey {
                        key: src_key.clone(),
                    }
                    .into_s3_error()
                })?
            };

            (src_obj.metadata.clone(), src_obj.version_id.clone())
        };

        // Determine destination versioning.
        let dst_bucket_ref = self
            .state
            .get_bucket(&dst_bucket)
            .map_err(S3ServiceError::into_s3_error)?;

        let dst_version_id = if dst_bucket_ref.is_versioning_enabled() {
            crate::utils::generate_version_id()
        } else {
            "null".to_owned()
        };

        // Drop the bucket ref before await to avoid holding it across await points.
        drop(dst_bucket_ref);

        // Copy storage data.
        let write_result = self
            .storage
            .copy_object(
                &src_bucket,
                &src_key,
                &src_version_for_storage,
                &dst_bucket,
                &dst_key,
                &dst_version_id,
            )
            .await
            .map_err(|e| S3ServiceError::Internal(anyhow::anyhow!("{e}")).into_s3_error())?;

        // Determine metadata: use source metadata unless MetadataDirective is REPLACE.
        let metadata = if req
            .input
            .metadata_directive
            .as_ref()
            .is_some_and(|d| d.as_str() == "REPLACE")
        {
            build_metadata_for_copy(&req.input, &req.headers)
        } else {
            src_metadata
        };

        let storage_class = req
            .input
            .storage_class
            .as_ref()
            .map_or_else(|| "STANDARD".to_owned(), |s| s.as_str().to_owned());

        let now = Utc::now();
        let dst_obj = S3Object {
            key: dst_key.clone(),
            version_id: dst_version_id.clone(),
            etag: write_result.etag.clone(),
            size: write_result.size,
            last_modified: now,
            storage_class,
            metadata,
            owner: InternalOwner::default(),
            checksum: None,
            parts_count: None,
            part_etags: Vec::new(),
        };

        // Re-acquire the bucket ref to store the object.
        let dst_bucket_ref = self
            .state
            .get_bucket(&dst_bucket)
            .map_err(S3ServiceError::into_s3_error)?;
        {
            let mut store = dst_bucket_ref.objects.write();
            store.put(dst_obj);
        }

        debug!(
            src_bucket = %src_bucket,
            src_key = %src_key,
            dst_bucket = %dst_bucket,
            dst_key = %dst_key,
            "copy_object completed"
        );

        let real_version_id = if dst_version_id == "null" {
            None
        } else {
            Some(dst_version_id)
        };

        let copy_result = CopyObjectResult {
            checksum_crc32: None,
            checksum_crc32c: None,
            checksum_crc64nvme: None,
            checksum_sha1: None,
            checksum_sha256: None,
            checksum_type: None,
            e_tag: Some(write_result.etag),
            last_modified: Some(chrono_to_timestamp(now)),
        };

        let output = CopyObjectOutput {
            bucket_key_enabled: None,
            copy_object_result: Some(copy_result),
            copy_source_version_id: src_version_id,
            expiration: None,
            request_charged: None,
            sse_customer_algorithm: None,
            sse_customer_key_md5: None,
            ssekms_encryption_context: None,
            ssekms_key_id: None,
            server_side_encryption: None,
            version_id: real_version_id,
        };
        Ok(S3Response::new(output))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect a streaming body into a single [`Bytes`] buffer.
pub(crate) async fn collect_body(body: Option<StreamingBlob>) -> S3Result<Bytes> {
    match body {
        Some(stream) => {
            let mut buf = BytesMut::new();
            let mut stream = stream;
            while let Some(chunk) = stream.try_next().await.map_err(|e| {
                let mut err = s3s::s3_error!(InternalError, "Failed to read body");
                err.set_source(e);
                err
            })? {
                buf.extend_from_slice(&chunk);
            }
            Ok(buf.freeze())
        }
        None => Ok(Bytes::new()),
    }
}

/// Build [`ObjectMetadata`] from a `PutObjectInput` and request headers.
fn build_metadata(input: &PutObjectInput, headers: &http::HeaderMap) -> ObjectMetadata {
    let user_metadata = input.metadata.clone().unwrap_or_default();

    // Parse tagging from the x-amz-tagging header.
    let tagging = input
        .tagging
        .as_ref()
        .map(|t| parse_tagging_header(t.as_str()))
        .unwrap_or_default();

    let acl = input
        .acl
        .as_ref()
        .and_then(|a| a.as_str().parse::<CannedAcl>().ok())
        .unwrap_or_default();

    ObjectMetadata {
        content_type: input
            .content_type
            .as_ref()
            .map(std::string::ToString::to_string),
        content_encoding: input.content_encoding.clone(),
        content_disposition: input.content_disposition.clone(),
        content_language: input.content_language.clone(),
        cache_control: input.cache_control.clone(),
        expires: input.expires.as_ref().map(|_| {
            headers
                .get("expires")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_owned()
        }),
        user_metadata,
        sse_algorithm: input
            .server_side_encryption
            .as_ref()
            .map(|s| s.as_str().to_owned()),
        sse_kms_key_id: input.ssekms_key_id.clone(),
        sse_bucket_key_enabled: input.bucket_key_enabled,
        sse_customer_algorithm: input.sse_customer_algorithm.clone(),
        sse_customer_key_md5: input.sse_customer_key_md5.clone(),
        tagging,
        acl,
        object_lock_mode: input
            .object_lock_mode
            .as_ref()
            .map(|m| m.as_str().to_owned()),
        object_lock_retain_until: input.object_lock_retain_until_date.as_ref().and_then(|ts| {
            let odt: time::OffsetDateTime = ts.clone().into();
            let unix_millis = odt.unix_timestamp() * 1000 + i64::from(odt.millisecond());
            chrono::DateTime::from_timestamp_millis(unix_millis)
        }),
        object_lock_legal_hold: input
            .object_lock_legal_hold_status
            .as_ref()
            .map(|s| s.as_str() == "ON"),
    }
}

/// Build [`ObjectMetadata`] for a copy operation with REPLACE directive.
fn build_metadata_for_copy(input: &CopyObjectInput, _headers: &http::HeaderMap) -> ObjectMetadata {
    let user_metadata = input.metadata.clone().unwrap_or_default();

    let tagging = input
        .tagging
        .as_ref()
        .map(|t| parse_tagging_header(t.as_str()))
        .unwrap_or_default();

    let acl = input
        .acl
        .as_ref()
        .and_then(|a| a.as_str().parse::<CannedAcl>().ok())
        .unwrap_or_default();

    ObjectMetadata {
        content_type: input
            .content_type
            .as_ref()
            .map(std::string::ToString::to_string),
        content_encoding: input.content_encoding.clone(),
        content_disposition: input.content_disposition.clone(),
        content_language: input.content_language.clone(),
        cache_control: input.cache_control.clone(),
        expires: None,
        user_metadata,
        sse_algorithm: input
            .server_side_encryption
            .as_ref()
            .map(|s| s.as_str().to_owned()),
        sse_kms_key_id: input.ssekms_key_id.clone(),
        sse_bucket_key_enabled: input.bucket_key_enabled,
        sse_customer_algorithm: input.sse_customer_algorithm.clone(),
        sse_customer_key_md5: input.sse_customer_key_md5.clone(),
        tagging,
        acl,
        object_lock_mode: input
            .object_lock_mode
            .as_ref()
            .map(|m| m.as_str().to_owned()),
        object_lock_retain_until: input.object_lock_retain_until_date.as_ref().and_then(|ts| {
            let odt: time::OffsetDateTime = ts.clone().into();
            let unix_millis = odt.unix_timestamp() * 1000 + i64::from(odt.millisecond());
            chrono::DateTime::from_timestamp_millis(unix_millis)
        }),
        object_lock_legal_hold: input
            .object_lock_legal_hold_status
            .as_ref()
            .map(|s| s.as_str() == "ON"),
    }
}

/// Parse the `x-amz-tagging` URL-encoded query string into tag pairs.
pub(super) fn parse_tagging_header(tagging: &str) -> Vec<(String, String)> {
    tagging
        .split('&')
        .filter(|s| !s.is_empty())
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            let key = percent_encoding::percent_decode_str(k)
                .decode_utf8()
                .ok()?
                .to_string();
            let value = percent_encoding::percent_decode_str(v)
                .decode_utf8()
                .ok()?
                .to_string();
            Some((key, value))
        })
        .collect()
}

/// Extract checksum data from a `PutObjectInput` if any checksum fields are set.
fn extract_checksum_from_put(input: &PutObjectInput) -> Option<ChecksumData> {
    if let Some(ref v) = input.checksum_crc32 {
        return Some(ChecksumData {
            algorithm: "CRC32".to_owned(),
            value: v.clone(),
        });
    }
    if let Some(ref v) = input.checksum_crc32c {
        return Some(ChecksumData {
            algorithm: "CRC32C".to_owned(),
            value: v.clone(),
        });
    }
    if let Some(ref v) = input.checksum_sha1 {
        return Some(ChecksumData {
            algorithm: "SHA1".to_owned(),
            value: v.clone(),
        });
    }
    if let Some(ref v) = input.checksum_sha256 {
        return Some(ChecksumData {
            algorithm: "SHA256".to_owned(),
            value: v.clone(),
        });
    }
    None
}
