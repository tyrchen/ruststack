//! Multipart upload operation handlers.
//!
//! Implements `create_multipart_upload`, `upload_part`, `upload_part_copy`,
//! `complete_multipart_upload`, `abort_multipart_upload`, `list_parts`,
//! and `list_multipart_uploads`.

use chrono::Utc;
// The s3s DTO module contains dozens of types we reference; wildcard is clearer.
#[allow(clippy::wildcard_imports)]
use s3s::dto::*;
use s3s::{S3Request, S3Response, S3Result};
use tracing::debug;

use crate::error::S3ServiceError;
use crate::provider::RustStackS3;
use crate::state::multipart::{MultipartUpload, UploadPart};
use crate::state::object::{ObjectMetadata, Owner as InternalOwner, S3Object};
use crate::utils::generate_upload_id;
use crate::validation::validate_object_key;

use super::bucket::chrono_to_timestamp;

// AWS S3 DTOs use signed integers (i32/i64) for inherently non-negative values.
// These handler methods must remain async to match the s3s::S3 trait interface.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unused_async
)]
impl RustStackS3 {
    /// Create a new multipart upload.
    #[allow(clippy::too_many_lines)]
    pub(crate) async fn handle_create_multipart_upload(
        &self,
        req: S3Request<CreateMultipartUploadInput>,
    ) -> S3Result<S3Response<CreateMultipartUploadOutput>> {
        let bucket_name = req.input.bucket;
        let key = req.input.key;

        validate_object_key(&key).map_err(S3ServiceError::into_s3_error)?;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let upload_id = generate_upload_id();

        // Build metadata from the request.
        let metadata = ObjectMetadata {
            content_type: req
                .input
                .content_type
                .as_ref()
                .map(std::string::ToString::to_string),
            content_encoding: req.input.content_encoding.clone(),
            content_disposition: req.input.content_disposition.clone(),
            content_language: req.input.content_language.clone(),
            cache_control: req.input.cache_control.clone(),
            expires: None,
            user_metadata: req.input.metadata.unwrap_or_default(),
            sse_algorithm: req
                .input
                .server_side_encryption
                .as_ref()
                .map(|s| s.as_str().to_owned()),
            sse_kms_key_id: req.input.ssekms_key_id.clone(),
            sse_bucket_key_enabled: req.input.bucket_key_enabled,
            sse_customer_algorithm: req.input.sse_customer_algorithm.clone(),
            sse_customer_key_md5: req.input.sse_customer_key_md5.clone(),
            tagging: req
                .input
                .tagging
                .as_ref()
                .map(|t| super::object::parse_tagging_header(t))
                .unwrap_or_default(),
            acl: req
                .input
                .acl
                .as_ref()
                .and_then(|a| a.as_str().parse().ok())
                .unwrap_or_default(),
            object_lock_mode: req
                .input
                .object_lock_mode
                .as_ref()
                .map(|m| m.as_str().to_owned()),
            object_lock_retain_until: req.input.object_lock_retain_until_date.as_ref().and_then(
                |ts| {
                    let odt: time::OffsetDateTime = ts.clone().into();
                    let unix_millis = odt.unix_timestamp() * 1000 + i64::from(odt.millisecond());
                    chrono::DateTime::from_timestamp_millis(unix_millis)
                },
            ),
            object_lock_legal_hold: req
                .input
                .object_lock_legal_hold_status
                .as_ref()
                .map(|s| s.as_str() == "ON"),
        };

        let mut upload = MultipartUpload::new(
            upload_id.clone(),
            key.clone(),
            InternalOwner::default(),
            metadata,
        );

        upload.storage_class = req
            .input
            .storage_class
            .as_ref()
            .map_or_else(|| "STANDARD".to_owned(), |s| s.as_str().to_owned());

        upload.checksum_algorithm = req
            .input
            .checksum_algorithm
            .as_ref()
            .map(|a| a.as_str().to_owned());

        upload.sse_algorithm = req
            .input
            .server_side_encryption
            .as_ref()
            .map(|s| s.as_str().to_owned());

        upload.sse_kms_key_id.clone_from(&req.input.ssekms_key_id);

        bucket.multipart_uploads.insert(upload_id.clone(), upload);

        debug!(
            bucket = %bucket_name,
            key = %key,
            upload_id = %upload_id,
            "create_multipart_upload completed"
        );

        let output = CreateMultipartUploadOutput {
            abort_date: None,
            abort_rule_id: None,
            bucket: Some(bucket_name),
            bucket_key_enabled: None,
            checksum_algorithm: req.input.checksum_algorithm,
            checksum_type: None,
            key: Some(key),
            request_charged: None,
            sse_customer_algorithm: None,
            sse_customer_key_md5: None,
            ssekms_encryption_context: None,
            ssekms_key_id: None,
            server_side_encryption: None,
            upload_id: Some(upload_id),
        };
        Ok(S3Response::new(output))
    }

    /// Upload a single part of a multipart upload.
    pub(crate) async fn handle_upload_part(
        &self,
        req: S3Request<UploadPartInput>,
    ) -> S3Result<S3Response<UploadPartOutput>> {
        let bucket_name = req.input.bucket;
        let key = req.input.key;
        let upload_id = req.input.upload_id;
        let part_number = req.input.part_number;

        if !(1..=10_000).contains(&part_number) {
            return Err(s3s::s3_error!(
                InvalidArgument,
                "Part number must be between 1 and 10000"
            ));
        }

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        // Verify the upload exists.
        let upload_ref = bucket.multipart_uploads.get(&upload_id).ok_or_else(|| {
            S3ServiceError::NoSuchUpload {
                upload_id: upload_id.clone(),
            }
            .into_s3_error()
        })?;
        // Drop the ref early; we only needed to verify existence.
        drop(upload_ref);

        // Collect body data.
        let body_data = super::object::collect_body(req.input.body).await?;

        // Write part to storage.
        let write_result = self
            .storage
            .write_part(&bucket_name, &upload_id, part_number as u32, body_data)
            .await
            .map_err(|e| S3ServiceError::Internal(anyhow::anyhow!("{e}")).into_s3_error())?;

        // Record the part metadata.
        let part = UploadPart {
            part_number: part_number as u32,
            etag: write_result.etag.clone(),
            size: write_result.size,
            last_modified: Utc::now(),
            checksum: None,
        };

        if let Some(mut upload) = bucket.multipart_uploads.get_mut(&upload_id) {
            upload.put_part(part);
        }

        debug!(
            bucket = %bucket_name,
            key = %key,
            upload_id = %upload_id,
            part_number,
            "upload_part completed"
        );

        let output = UploadPartOutput {
            bucket_key_enabled: None,
            checksum_crc32: None,
            checksum_crc32c: None,
            checksum_crc64nvme: None,
            checksum_sha1: None,
            checksum_sha256: None,
            e_tag: Some(write_result.etag),
            request_charged: None,
            sse_customer_algorithm: None,
            sse_customer_key_md5: None,
            ssekms_key_id: None,
            server_side_encryption: None,
        };
        Ok(S3Response::new(output))
    }

    /// Upload a part by copying from an existing object.
    pub(crate) async fn handle_upload_part_copy(
        &self,
        req: S3Request<UploadPartCopyInput>,
    ) -> S3Result<S3Response<UploadPartCopyOutput>> {
        let bucket_name = req.input.bucket;
        let upload_id = req.input.upload_id;
        let part_number = req.input.part_number;

        let copy_source = req.input.copy_source;

        let (src_bucket, src_key, src_version_id) = match copy_source {
            CopySource::Bucket {
                bucket,
                key,
                version_id,
            } => (
                bucket.to_string(),
                key.to_string(),
                version_id.map(|v| v.to_string()),
            ),
            CopySource::AccessPoint { .. } => {
                return Err(s3s::s3_error!(
                    NotImplemented,
                    "AccessPoint copy source is not supported"
                ));
            }
        };

        // Read source object data.
        let src_vid = src_version_id.as_deref().unwrap_or("null");
        let data = self
            .storage
            .read_object(&src_bucket, &src_key, src_vid, None)
            .await
            .map_err(|e| S3ServiceError::Internal(anyhow::anyhow!("{e}")).into_s3_error())?;

        // Write as part.
        let write_result = self
            .storage
            .write_part(&bucket_name, &upload_id, part_number as u32, data)
            .await
            .map_err(|e| S3ServiceError::Internal(anyhow::anyhow!("{e}")).into_s3_error())?;

        // Record the part metadata.
        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let part = UploadPart {
            part_number: part_number as u32,
            etag: write_result.etag.clone(),
            size: write_result.size,
            last_modified: Utc::now(),
            checksum: None,
        };

        if let Some(mut upload) = bucket.multipart_uploads.get_mut(&upload_id) {
            upload.put_part(part);
        }

        let copy_result = CopyPartResult {
            checksum_crc32: None,
            checksum_crc32c: None,
            checksum_crc64nvme: None,
            checksum_sha1: None,
            checksum_sha256: None,
            e_tag: Some(write_result.etag),
            last_modified: Some(chrono_to_timestamp(Utc::now())),
        };

        let output = UploadPartCopyOutput {
            bucket_key_enabled: None,
            copy_part_result: Some(copy_result),
            copy_source_version_id: src_version_id,
            request_charged: None,
            sse_customer_algorithm: None,
            sse_customer_key_md5: None,
            ssekms_key_id: None,
            server_side_encryption: None,
        };
        Ok(S3Response::new(output))
    }

    /// Complete a multipart upload by assembling parts into the final object.
    #[allow(clippy::too_many_lines)]
    pub(crate) async fn handle_complete_multipart_upload(
        &self,
        req: S3Request<CompleteMultipartUploadInput>,
    ) -> S3Result<S3Response<CompleteMultipartUploadOutput>> {
        let bucket_name = req.input.bucket;
        let key = req.input.key;
        let upload_id = req.input.upload_id;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        // Get the upload.
        let upload = bucket
            .multipart_uploads
            .get(&upload_id)
            .ok_or_else(|| {
                S3ServiceError::NoSuchUpload {
                    upload_id: upload_id.clone(),
                }
                .into_s3_error()
            })?
            .clone();

        // Extract the requested part list.
        let multipart_upload = req.input.multipart_upload;
        let requested_parts = multipart_upload.and_then(|mu| mu.parts).unwrap_or_default();

        // Validate parts are in order and exist.
        let mut part_numbers: Vec<u32> = Vec::with_capacity(requested_parts.len());
        let mut last_num = 0i32;

        for cp in &requested_parts {
            let part_num = cp
                .part_number
                .ok_or_else(|| s3s::s3_error!(InvalidArgument, "Part number is required"))?;

            if part_num <= last_num {
                return Err(S3ServiceError::InvalidPartOrder.into_s3_error());
            }
            last_num = part_num;

            let part_num_u32 = u32::try_from(part_num)
                .map_err(|_| s3s::s3_error!(InvalidArgument, "Invalid part number"))?;

            // Verify the part exists in our upload record.
            upload
                .get_part(part_num_u32)
                .ok_or_else(|| S3ServiceError::InvalidPart.into_s3_error())?;

            part_numbers.push(part_num_u32);
        }

        // Determine version ID.
        let version_id = if bucket.is_versioning_enabled() {
            crate::utils::generate_version_id()
        } else {
            "null".to_owned()
        };

        // Assemble parts in storage.
        let (write_result, _part_md5s) = self
            .storage
            .complete_multipart(&bucket_name, &upload_id, &key, &version_id, &part_numbers)
            .await
            .map_err(|e| S3ServiceError::Internal(anyhow::anyhow!("{e}")).into_s3_error())?;

        // Build the final object.
        let obj = S3Object {
            key: key.clone(),
            version_id: version_id.clone(),
            etag: write_result.etag.clone(),
            size: write_result.size,
            last_modified: Utc::now(),
            storage_class: upload.storage_class.clone(),
            metadata: upload.metadata.clone(),
            owner: upload.owner.clone(),
            checksum: None,
            parts_count: Some(part_numbers.len() as u32),
            part_etags: requested_parts
                .iter()
                .filter_map(|p| p.e_tag.clone())
                .collect(),
        };

        {
            let mut store = bucket.objects.write();
            store.put(obj);
        }

        // Remove the completed upload.
        bucket.multipart_uploads.remove(&upload_id);

        debug!(
            bucket = %bucket_name,
            key = %key,
            upload_id = %upload_id,
            parts = part_numbers.len(),
            "complete_multipart_upload completed"
        );

        let real_version_id = if version_id == "null" {
            None
        } else {
            Some(version_id)
        };

        let output = CompleteMultipartUploadOutput {
            bucket: Some(bucket_name.clone()),
            bucket_key_enabled: None,
            checksum_crc32: None,
            checksum_crc32c: None,
            checksum_crc64nvme: None,
            checksum_sha1: None,
            checksum_sha256: None,
            checksum_type: None,
            e_tag: Some(write_result.etag),
            expiration: None,
            key: Some(key),
            location: Some(format!("http://s3.amazonaws.com/{bucket_name}")),
            request_charged: None,
            ssekms_key_id: None,
            server_side_encryption: None,
            version_id: real_version_id,
        };
        Ok(S3Response::new(output))
    }

    /// Abort a multipart upload.
    pub(crate) async fn handle_abort_multipart_upload(
        &self,
        req: S3Request<AbortMultipartUploadInput>,
    ) -> S3Result<S3Response<AbortMultipartUploadOutput>> {
        let bucket_name = req.input.bucket;
        let upload_id = req.input.upload_id;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        // Remove the upload metadata.
        bucket.multipart_uploads.remove(&upload_id).ok_or_else(|| {
            S3ServiceError::NoSuchUpload {
                upload_id: upload_id.clone(),
            }
            .into_s3_error()
        })?;

        // Clean up storage parts.
        self.storage.abort_multipart(&bucket_name, &upload_id);

        debug!(
            bucket = %bucket_name,
            upload_id = %upload_id,
            "abort_multipart_upload completed"
        );

        let output = AbortMultipartUploadOutput {
            request_charged: None,
        };
        Ok(S3Response::new(output))
    }

    /// List parts that have been uploaded for a multipart upload.
    pub(crate) async fn handle_list_parts(
        &self,
        req: S3Request<ListPartsInput>,
    ) -> S3Result<S3Response<ListPartsOutput>> {
        let bucket_name = req.input.bucket;
        let key = req.input.key;
        let upload_id = req.input.upload_id;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let upload = bucket.multipart_uploads.get(&upload_id).ok_or_else(|| {
            S3ServiceError::NoSuchUpload {
                upload_id: upload_id.clone(),
            }
            .into_s3_error()
        })?;

        let max_parts = req.input.max_parts.unwrap_or(1000) as usize;
        let part_number_marker: u32 = req
            .input
            .part_number_marker
            .as_deref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let all_parts: Vec<&UploadPart> = upload
            .parts
            .values()
            .filter(|p| p.part_number > part_number_marker)
            .collect();

        let is_truncated = all_parts.len() > max_parts;
        let parts_to_return = &all_parts[..all_parts.len().min(max_parts)];

        let s3_parts: Vec<Part> = parts_to_return
            .iter()
            .map(|p| Part {
                checksum_crc32: None,
                checksum_crc32c: None,
                checksum_crc64nvme: None,
                checksum_sha1: None,
                checksum_sha256: None,
                e_tag: Some(p.etag.clone()),
                last_modified: Some(chrono_to_timestamp(p.last_modified)),
                part_number: Some(p.part_number as i32),
                size: Some(p.size as i64),
            })
            .collect();

        let next_marker = if is_truncated {
            s3_parts.last().and_then(|p| p.part_number)
        } else {
            None
        };

        let owner = super::bucket::to_s3_owner(&upload.owner);

        let output = ListPartsOutput {
            abort_date: None,
            abort_rule_id: None,
            bucket: Some(bucket_name),
            checksum_algorithm: upload
                .checksum_algorithm
                .as_ref()
                .map(|a| ChecksumAlgorithm::from(a.clone())),
            checksum_type: None,
            initiator: Some(Initiator {
                display_name: Some(upload.owner.display_name.clone()),
                id: Some(upload.owner.id.clone()),
            }),
            is_truncated: Some(is_truncated),
            key: Some(key),
            max_parts: Some(max_parts as i32),
            next_part_number_marker: next_marker.map(|n| n.to_string()),
            owner: Some(owner),
            part_number_marker: Some(part_number_marker.to_string()),
            parts: Some(s3_parts),
            request_charged: None,
            storage_class: Some(StorageClass::from(upload.storage_class.clone())),
            upload_id: Some(upload_id),
        };
        Ok(S3Response::new(output))
    }

    /// List in-progress multipart uploads for a bucket.
    pub(crate) async fn handle_list_multipart_uploads(
        &self,
        req: S3Request<ListMultipartUploadsInput>,
    ) -> S3Result<S3Response<ListMultipartUploadsOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let prefix = req.input.prefix.unwrap_or_default();
        let max_uploads = req.input.max_uploads.unwrap_or(1000) as usize;

        let mut uploads: Vec<MultipartUpload> = bucket
            .multipart_uploads
            .iter()
            .filter(|entry| entry.key.starts_with(&prefix))
            .map(|entry| entry.value().clone())
            .collect();

        // Sort by key then by initiated time.
        uploads.sort_by(|a, b| a.key.cmp(&b.key).then(a.initiated.cmp(&b.initiated)));

        let is_truncated = uploads.len() > max_uploads;
        let uploads_to_return = &uploads[..uploads.len().min(max_uploads)];

        let s3_uploads: Vec<s3s::dto::MultipartUpload> = uploads_to_return
            .iter()
            .map(|u| s3s::dto::MultipartUpload {
                checksum_algorithm: u
                    .checksum_algorithm
                    .as_ref()
                    .map(|a| ChecksumAlgorithm::from(a.clone())),
                checksum_type: None,
                initiated: Some(chrono_to_timestamp(u.initiated)),
                initiator: Some(Initiator {
                    display_name: Some(u.owner.display_name.clone()),
                    id: Some(u.owner.id.clone()),
                }),
                key: Some(u.key.clone()),
                owner: Some(super::bucket::to_s3_owner(&u.owner)),
                storage_class: Some(StorageClass::from(u.storage_class.clone())),
                upload_id: Some(u.upload_id.clone()),
            })
            .collect();

        let next_key_marker = if is_truncated {
            s3_uploads.last().and_then(|u| u.key.clone())
        } else {
            None
        };
        let next_upload_id_marker = if is_truncated {
            s3_uploads.last().and_then(|u| u.upload_id.clone())
        } else {
            None
        };

        let output = ListMultipartUploadsOutput {
            bucket: Some(bucket_name),
            common_prefixes: None,
            delimiter: req.input.delimiter,
            encoding_type: None,
            is_truncated: Some(is_truncated),
            key_marker: req.input.key_marker,
            max_uploads: Some(max_uploads as i32),
            next_key_marker,
            next_upload_id_marker,
            prefix: Some(prefix),
            request_charged: None,
            upload_id_marker: req.input.upload_id_marker,
            uploads: Some(s3_uploads),
        };
        Ok(S3Response::new(output))
    }
}
