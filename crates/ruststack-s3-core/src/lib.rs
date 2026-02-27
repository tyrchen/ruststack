//! S3 service implementation for RustStack, built on the s3s crate.
//!
//! This crate implements the [`s3s::S3`] trait to provide a fully-featured,
//! in-memory S3 service compatible with LocalStack. It supports bucket CRUD,
//! object CRUD, multipart uploads, versioning, CORS, tagging, ACLs, encryption
//! metadata, checksums, object lock, and more.
//!
//! # Architecture
//!
//! ```text
//! s3s HTTP Layer (routing, XML, auth)
//!        |
//!        v
//! RustStackS3 (s3s::S3 trait impl)
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
mod ops;
pub mod provider;
pub mod state;
pub mod storage;
pub mod utils;
pub mod validation;

pub use config::S3Config;
pub use provider::RustStackS3;
