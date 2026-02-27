//! S3 operation handlers.
//!
//! This module contains the implementations of all S3 operations, organized
//! into submodules by category. Each submodule exposes `handle_*` methods
//! on [`crate::provider::RustStackS3`].
//!
//! The server binary bridges these handlers to the HTTP layer by implementing
//! the `S3Handler` trait from `ruststack-s3-http`.

pub mod bucket;
pub mod bucket_config;
pub mod list;
pub mod multipart;
pub mod object;
pub mod object_config;
