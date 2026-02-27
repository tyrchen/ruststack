//! AWS Signature Version 4 request authentication for RustStack S3.
//!
//! This crate provides SigV4 signature verification for incoming HTTP requests
//! to an S3-compatible server. It supports both header-based authentication
//! (via the `Authorization` header) and presigned URL authentication (via query
//! parameters).
//!
//! # Overview
//!
//! AWS Signature Version 4 is the standard authentication mechanism for AWS API
//! requests. This crate implements the verification side: given an incoming HTTP
//! request and a credential store, it verifies that the request was signed by a
//! known access key with the correct secret key.
//!
//! # Usage
//!
//! ```rust
//! use ruststack_s3_auth::credentials::{CredentialProvider, StaticCredentialProvider};
//! use ruststack_s3_auth::sigv4::{hash_payload, verify_sigv4};
//!
//! // Set up credentials
//! let provider = StaticCredentialProvider::new(vec![
//!     ("AKIAIOSFODNN7EXAMPLE".to_owned(), "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_owned()),
//! ]);
//!
//! // For header-based auth, call verify_sigv4 with the request parts and body hash.
//! // For presigned URLs, call verify_presigned with the request parts.
//! ```
//!
//! # Modules
//!
//! - [`canonical`] - Canonical request construction per the SigV4 specification
//! - [`credentials`] - Credential provider trait and in-memory implementation
//! - [`error`] - Authentication error types
//! - [`presigned`] - Presigned URL verification
//! - [`sigv4`] - Main SigV4 signature verification logic

pub mod canonical;
pub mod credentials;
pub mod error;
pub mod presigned;
pub mod sigv4;

pub use credentials::{CredentialProvider, StaticCredentialProvider};
pub use error::AuthError;
pub use presigned::verify_presigned;
pub use sigv4::{AuthResult, hash_payload, verify_sigv4};
