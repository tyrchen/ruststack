//! S3 service implementation for RustStack.
//!
//! This crate implements the S3 business logic provider (`RustStackS3`) that can
//! be plugged into the `ruststack-s3-http` service layer via the `S3Handler` trait.
//! It supports bucket CRUD, object CRUD, multipart uploads, versioning, CORS,
//! tagging, ACLs, encryption metadata, checksums, object lock, and more.
//!
//! # Architecture
//!
//! ```text
//! ruststack-s3-http (routing, XML, auth)
//!        |
//!        v
//! RustStackS3 (S3Handler trait impl in server)
//!        |
//!        v
//!   S3ServiceState (buckets, global index)
//!        |
//!        v
//!   StorageBackend (in-memory + spillover)
//! ```

pub mod auth;
pub mod checksums;
pub mod config;
pub mod cors;
pub mod error;
pub mod ops;
pub mod provider;
pub mod state;
pub mod storage;
pub mod utils;
pub mod validation;

pub use config::S3Config;
pub use provider::RustStackS3;
