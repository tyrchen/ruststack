//! SQS HTTP service layer for Rustack.
//!
//! This crate implements the HTTP transport for SQS, handling:
//! - Request routing via `X-Amz-Target: AmazonSQS.*` header
//! - JSON request/response serialization (`awsJson1_0` protocol)
//! - Error formatting with `x-amzn-query-error` compatibility header
//! - Optional SigV4 authentication

pub mod body;
pub mod dispatch;
pub mod response;
pub mod router;
pub mod service;
