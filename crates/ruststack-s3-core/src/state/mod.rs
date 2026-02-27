//! S3 service state management.
//!
//! This module provides the in-memory state for the S3 service:
//!
//! - [`S3ServiceState`] -- top-level service owning all buckets
//! - [`S3Bucket`] -- per-bucket state (objects, versioning, configs)
//! - [`ObjectStore`] / [`KeyStore`] / [`VersionedKeyStore`] -- key-level storage
//! - [`S3Object`] / [`S3DeleteMarker`] / [`ObjectMetadata`] -- object types
//! - [`MultipartUpload`] / [`UploadPart`] -- multipart upload tracking
//!
//! # Thread Safety
//!
//! All types are `Send + Sync`. Concurrent access is handled via:
//!
//! - `DashMap` for the bucket table and multipart upload table
//! - `parking_lot::RwLock` for per-bucket configuration fields and the object
//!   store

pub(crate) mod bucket;
pub(crate) mod keystore;
pub(crate) mod multipart;
pub(crate) mod object;
pub(crate) mod service;

pub use bucket::{
    BucketEncryption, CorsRuleConfig, DefaultRetention, ObjectLockConfiguration, ObjectLockRule,
    OwnershipControlsConfig, PublicAccessBlockConfig, S3Bucket, VersioningStatus,
};
pub use keystore::{
    KeyStore, ListResult, ObjectStore, VersionListEntry, VersionListResult, VersionedKeyStore,
};
pub use multipart::{MultipartUpload, UploadPart};
pub use object::{
    CannedAcl, ChecksumData, Grant, Grantee, ObjectMetadata, ObjectVersion, Owner, Permission,
    S3DeleteMarker, S3Object,
};
pub use service::S3ServiceState;
