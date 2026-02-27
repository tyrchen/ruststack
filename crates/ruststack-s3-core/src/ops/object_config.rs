//! Object configuration operation handlers.
//!
//! Implements `get_object_tagging`, `put_object_tagging`, `delete_object_tagging`,
//! `get_object_acl`, `put_object_acl`, `get_object_retention`,
//! `put_object_retention`, `get_object_legal_hold`, `put_object_legal_hold`,
//! and `get_object_attributes`.

// The s3s DTO module contains dozens of types we reference; wildcard is clearer.
#[allow(clippy::wildcard_imports)]
use s3s::dto::*;
use s3s::{S3Request, S3Response, S3Result};
use tracing::debug;

use crate::error::S3ServiceError;
use crate::provider::RustStackS3;
use crate::state::object::CannedAcl;

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
    // -----------------------------------------------------------------------
    // Object Tagging
    // -----------------------------------------------------------------------

    /// Get tags for an object.
    pub(crate) async fn handle_get_object_tagging(
        &self,
        req: S3Request<GetObjectTaggingInput>,
    ) -> S3Result<S3Response<GetObjectTaggingOutput>> {
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

        let tag_set: Vec<Tag> = obj
            .metadata
            .tagging
            .iter()
            .map(|(k, v)| Tag {
                key: Some(k.clone()),
                value: Some(v.clone()),
            })
            .collect();

        let version_id = if obj.version_id == "null" {
            None
        } else {
            Some(obj.version_id.clone())
        };

        let output = GetObjectTaggingOutput {
            tag_set,
            version_id,
        };
        Ok(S3Response::new(output))
    }

    /// Set tags for an object.
    pub(crate) async fn handle_put_object_tagging(
        &self,
        req: S3Request<PutObjectTaggingInput>,
    ) -> S3Result<S3Response<PutObjectTaggingOutput>> {
        let bucket_name = req.input.bucket;
        let key = req.input.key;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let tagging = req.input.tagging;

        let tags: Vec<(String, String)> = tagging
            .tag_set
            .into_iter()
            .map(|t| (t.key.unwrap_or_default(), t.value.unwrap_or_default()))
            .collect();

        crate::validation::validate_tags(&tags).map_err(S3ServiceError::into_s3_error)?;

        // We need mutable access to the object. Since ObjectStore wraps objects
        // immutably, we re-insert a modified copy.
        let mut store = bucket.objects.write();
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

        let mut updated = obj.clone();
        updated.metadata.tagging = tags;
        store.put(updated);

        debug!(bucket = %bucket_name, key = %key, "put_object_tagging completed");

        let version_id_out = req.input.version_id.clone();
        let output = PutObjectTaggingOutput {
            version_id: version_id_out,
        };
        Ok(S3Response::new(output))
    }

    /// Delete tags for an object.
    pub(crate) async fn handle_delete_object_tagging(
        &self,
        req: S3Request<DeleteObjectTaggingInput>,
    ) -> S3Result<S3Response<DeleteObjectTaggingOutput>> {
        let bucket_name = req.input.bucket;
        let key = req.input.key;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let mut store = bucket.objects.write();
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

        let mut updated = obj.clone();
        updated.metadata.tagging = Vec::new();
        store.put(updated);

        debug!(bucket = %bucket_name, key = %key, "delete_object_tagging completed");

        let version_id_out = req.input.version_id.clone();
        let output = DeleteObjectTaggingOutput {
            version_id: version_id_out,
        };
        Ok(S3Response::new(output))
    }

    // -----------------------------------------------------------------------
    // Object ACL
    // -----------------------------------------------------------------------

    /// Get the ACL for an object.
    pub(crate) async fn handle_get_object_acl(
        &self,
        req: S3Request<GetObjectAclInput>,
    ) -> S3Result<S3Response<GetObjectAclOutput>> {
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

        let owner = super::bucket::to_s3_owner(&obj.owner);

        let grant = Grant {
            grantee: Some(Grantee {
                display_name: Some(obj.owner.display_name.clone()),
                email_address: None,
                id: Some(obj.owner.id.clone()),
                type_: s3s::dto::Type::from_static("CanonicalUser"),
                uri: None,
            }),
            permission: Some(Permission::from_static("FULL_CONTROL")),
        };

        let output = GetObjectAclOutput {
            grants: Some(vec![grant]),
            owner: Some(owner),
            request_charged: None,
        };
        Ok(S3Response::new(output))
    }

    /// Set the ACL for an object.
    pub(crate) async fn handle_put_object_acl(
        &self,
        req: S3Request<PutObjectAclInput>,
    ) -> S3Result<S3Response<PutObjectAclOutput>> {
        let bucket_name = req.input.bucket;
        let key = req.input.key;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        if let Some(acl_str) = req.input.acl {
            let acl: CannedAcl = acl_str
                .as_str()
                .parse()
                .map_err(|_| s3s::s3_error!(InvalidArgument, "Invalid canned ACL"))?;

            let mut store = bucket.objects.write();
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

            let mut updated = obj.clone();
            updated.metadata.acl = acl;
            store.put(updated);
        }

        debug!(bucket = %bucket_name, key = %key, "put_object_acl completed");

        let output = PutObjectAclOutput {
            request_charged: None,
        };
        Ok(S3Response::new(output))
    }

    // -----------------------------------------------------------------------
    // Object Retention
    // -----------------------------------------------------------------------

    /// Get the retention configuration for an object.
    pub(crate) async fn handle_get_object_retention(
        &self,
        req: S3Request<GetObjectRetentionInput>,
    ) -> S3Result<S3Response<GetObjectRetentionOutput>> {
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

        let retention = match (
            &obj.metadata.object_lock_mode,
            obj.metadata.object_lock_retain_until,
        ) {
            (Some(mode), Some(until)) => Some(ObjectLockRetention {
                mode: Some(ObjectLockRetentionMode::from(mode.clone())),
                retain_until_date: Some(chrono_to_timestamp(until)),
            }),
            _ => None,
        };

        if retention.is_none() {
            return Err(s3s::s3_error!(
                InvalidArgument,
                "No retention configuration found"
            ));
        }

        let output = GetObjectRetentionOutput { retention };
        Ok(S3Response::new(output))
    }

    /// Set the retention configuration for an object.
    pub(crate) async fn handle_put_object_retention(
        &self,
        req: S3Request<PutObjectRetentionInput>,
    ) -> S3Result<S3Response<PutObjectRetentionOutput>> {
        let bucket_name = req.input.bucket;
        let key = req.input.key;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let retention = req.input.retention;

        let mut store = bucket.objects.write();
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

        let mut updated = obj.clone();
        if let Some(ret) = retention {
            updated.metadata.object_lock_mode = ret.mode.as_ref().map(|m| m.as_str().to_owned());
            updated.metadata.object_lock_retain_until = ret.retain_until_date.and_then(|ts| {
                let odt: time::OffsetDateTime = ts.into();
                let unix_millis = odt.unix_timestamp() * 1000 + i64::from(odt.millisecond());
                chrono::DateTime::from_timestamp_millis(unix_millis)
            });
        } else {
            updated.metadata.object_lock_mode = None;
            updated.metadata.object_lock_retain_until = None;
        }
        store.put(updated);

        debug!(bucket = %bucket_name, key = %key, "put_object_retention completed");

        let output = PutObjectRetentionOutput {
            request_charged: None,
        };
        Ok(S3Response::new(output))
    }

    // -----------------------------------------------------------------------
    // Object Legal Hold
    // -----------------------------------------------------------------------

    /// Get the legal hold status for an object.
    pub(crate) async fn handle_get_object_legal_hold(
        &self,
        req: S3Request<GetObjectLegalHoldInput>,
    ) -> S3Result<S3Response<GetObjectLegalHoldOutput>> {
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

        let is_on = obj.metadata.object_lock_legal_hold.unwrap_or(false);
        let status = if is_on {
            ObjectLockLegalHoldStatus::from_static("ON")
        } else {
            ObjectLockLegalHoldStatus::from_static("OFF")
        };

        let output = GetObjectLegalHoldOutput {
            legal_hold: Some(ObjectLockLegalHold {
                status: Some(status),
            }),
        };
        Ok(S3Response::new(output))
    }

    /// Set the legal hold status for an object.
    pub(crate) async fn handle_put_object_legal_hold(
        &self,
        req: S3Request<PutObjectLegalHoldInput>,
    ) -> S3Result<S3Response<PutObjectLegalHoldOutput>> {
        let bucket_name = req.input.bucket;
        let key = req.input.key;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let legal_hold = req.input.legal_hold;

        let mut store = bucket.objects.write();
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

        let mut updated = obj.clone();
        updated.metadata.object_lock_legal_hold = legal_hold
            .and_then(|lh| lh.status)
            .map(|s| s.as_str() == "ON");
        store.put(updated);

        debug!(bucket = %bucket_name, key = %key, "put_object_legal_hold completed");

        let output = PutObjectLegalHoldOutput {
            request_charged: None,
        };
        Ok(S3Response::new(output))
    }

    // -----------------------------------------------------------------------
    // Get Object Attributes
    // -----------------------------------------------------------------------

    /// Get attributes for an object.
    pub(crate) async fn handle_get_object_attributes(
        &self,
        req: S3Request<GetObjectAttributesInput>,
    ) -> S3Result<S3Response<GetObjectAttributesOutput>> {
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

        let version_id = if obj.version_id == "null" {
            None
        } else {
            Some(obj.version_id.clone())
        };

        let output = GetObjectAttributesOutput {
            checksum: None,
            delete_marker: None,
            e_tag: Some(obj.etag.clone()),
            last_modified: Some(chrono_to_timestamp(obj.last_modified)),
            object_parts: obj.parts_count.map(|n| GetObjectAttributesParts {
                is_truncated: None,
                max_parts: None,
                next_part_number_marker: None,
                part_number_marker: None,
                parts: None,
                total_parts_count: Some(n as i32),
            }),
            object_size: Some(obj.size as i64),
            request_charged: None,
            storage_class: Some(StorageClass::from(obj.storage_class.clone())),
            version_id,
        };
        Ok(S3Response::new(output))
    }
}
