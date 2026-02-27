//! Bucket CRUD operation handlers.
//!
//! Implements `create_bucket`, `delete_bucket`, `head_bucket`, `list_buckets`,
//! and `get_bucket_location`.

// The s3s DTO module contains dozens of types we reference; wildcard is clearer.
#[allow(clippy::wildcard_imports)]
use s3s::dto::*;
use s3s::{S3Request, S3Response, S3Result};
use tracing::debug;

use crate::error::S3ServiceError;
use crate::provider::RustStackS3;
use crate::state::object::Owner as InternalOwner;
use crate::validation::validate_bucket_name;

/// Convert our internal [`InternalOwner`] to the s3s [`Owner`] DTO.
pub(super) fn to_s3_owner(owner: &InternalOwner) -> Owner {
    Owner {
        display_name: Some(owner.display_name.clone()),
        id: Some(owner.id.clone()),
    }
}

// These handler methods must remain async to match the s3s::S3 trait interface,
// even when the method body is synchronous.
#[allow(clippy::unused_async)]
impl RustStackS3 {
    /// Create a new S3 bucket.
    pub(crate) async fn handle_create_bucket(
        &self,
        req: S3Request<CreateBucketInput>,
    ) -> S3Result<S3Response<CreateBucketOutput>> {
        let bucket_name = req.input.bucket;

        validate_bucket_name(&bucket_name).map_err(S3ServiceError::into_s3_error)?;

        let region = req
            .input
            .create_bucket_configuration
            .and_then(|c| c.location_constraint)
            .map_or_else(
                || self.config.default_region.clone(),
                |lc| lc.as_str().to_owned(),
            );

        let account_id = req
            .credentials
            .as_ref()
            .map_or_else(|| InternalOwner::default().id, |c| c.access_key.clone());

        let owner = InternalOwner {
            id: account_id.clone(),
            display_name: account_id,
        };

        // Check if object lock is requested.
        let object_lock_enabled = req.input.object_lock_enabled_for_bucket.unwrap_or(false);

        self.state
            .create_bucket(bucket_name.clone(), region, owner)
            .map_err(S3ServiceError::into_s3_error)?;

        // If object lock was requested, enable it on the bucket.
        if object_lock_enabled {
            if let Ok(bucket) = self.state.get_bucket(&bucket_name) {
                *bucket.object_lock_enabled.write() = true;
                // Object lock requires versioning.
                bucket.enable_versioning();
            }
        }

        debug!(bucket = %bucket_name, "create_bucket completed");

        let output = CreateBucketOutput {
            location: Some(format!("/{bucket_name}")),
        };
        Ok(S3Response::new(output))
    }

    /// Delete an S3 bucket.
    pub(crate) async fn handle_delete_bucket(
        &self,
        req: S3Request<DeleteBucketInput>,
    ) -> S3Result<S3Response<DeleteBucketOutput>> {
        let bucket_name = req.input.bucket;

        // Clean up CORS rules for this bucket.
        self.cors_index.delete_rules(&bucket_name);

        // Delete storage data for this bucket.
        self.storage.delete_bucket_data(&bucket_name);

        // Delete the bucket from state.
        self.state
            .delete_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        debug!(bucket = %bucket_name, "delete_bucket completed");

        Ok(S3Response::new(DeleteBucketOutput {}))
    }

    /// Check if a bucket exists and is accessible (HEAD Bucket).
    pub(crate) async fn handle_head_bucket(
        &self,
        req: S3Request<HeadBucketInput>,
    ) -> S3Result<S3Response<HeadBucketOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let output = HeadBucketOutput {
            access_point_alias: None,
            bucket_location_name: Some(bucket.region.clone()),
            bucket_location_type: Some(LocationType::from_static("Region")),
            bucket_region: Some(bucket.region.clone()),
        };
        Ok(S3Response::new(output))
    }

    /// List all buckets.
    pub(crate) async fn handle_list_buckets(
        &self,
        _req: S3Request<ListBucketsInput>,
    ) -> S3Result<S3Response<ListBucketsOutput>> {
        let bucket_list = self.state.list_buckets();

        let buckets: Vec<Bucket> = bucket_list
            .into_iter()
            .map(|(name, creation_date)| {
                let ts = chrono_to_timestamp(creation_date);
                Bucket {
                    name: Some(name),
                    creation_date: Some(ts),
                    bucket_region: None,
                }
            })
            .collect();

        let owner = to_s3_owner(&InternalOwner::default());

        let output = ListBucketsOutput {
            buckets: Some(buckets),
            continuation_token: None,
            owner: Some(owner),
            prefix: None,
        };
        Ok(S3Response::new(output))
    }

    /// Get the location (region) of a bucket.
    pub(crate) async fn handle_get_bucket_location(
        &self,
        req: S3Request<GetBucketLocationInput>,
    ) -> S3Result<S3Response<GetBucketLocationOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let location_constraint = if bucket.region == "us-east-1" {
            // AWS returns null/empty for us-east-1.
            None
        } else {
            Some(BucketLocationConstraint::from(bucket.region.clone()))
        };

        let output = GetBucketLocationOutput {
            location_constraint,
        };
        Ok(S3Response::new(output))
    }
}

/// Convert a `chrono::DateTime<Utc>` to an s3s [`Timestamp`].
pub(crate) fn chrono_to_timestamp(dt: chrono::DateTime<chrono::Utc>) -> Timestamp {
    let system_time = std::time::SystemTime::UNIX_EPOCH
        + std::time::Duration::from_millis(
            u64::try_from(dt.timestamp_millis()).unwrap_or_default(),
        );
    Timestamp::from(system_time)
}
