//! Auto-generated from AWS S3 Smithy model. DO NOT EDIT.

use crate::types::{Bucket, BucketLocationConstraint, LocationType, Owner};

/// S3 CreateBucketOutput.
#[derive(Debug, Clone, Default)]
pub struct CreateBucketOutput {
    /// HTTP header: `x-amz-bucket-arn`.
    pub bucket_arn: Option<String>,
    /// HTTP header: `Location`.
    pub location: Option<String>,
}

/// S3 GetBucketLocationOutput.
#[derive(Debug, Clone, Default)]
pub struct GetBucketLocationOutput {
    pub location_constraint: Option<BucketLocationConstraint>,
}

/// S3 HeadBucketOutput.
#[derive(Debug, Clone, Default)]
pub struct HeadBucketOutput {
    /// HTTP header: `x-amz-access-point-alias`.
    pub access_point_alias: Option<bool>,
    /// HTTP header: `x-amz-bucket-arn`.
    pub bucket_arn: Option<String>,
    /// HTTP header: `x-amz-bucket-location-name`.
    pub bucket_location_name: Option<String>,
    /// HTTP header: `x-amz-bucket-location-type`.
    pub bucket_location_type: Option<LocationType>,
    /// HTTP header: `x-amz-bucket-region`.
    pub bucket_region: Option<String>,
}

/// S3 ListBucketsOutput.
#[derive(Debug, Clone, Default)]
pub struct ListBucketsOutput {
    pub buckets: Vec<Bucket>,
    pub continuation_token: Option<String>,
    pub owner: Option<Owner>,
    pub prefix: Option<String>,
}
