//! SES HTTP service layer for RustStack.
//!
//! SES v1 uses the `awsQuery` protocol (same as SNS): requests are
//! `application/x-www-form-urlencoded` with `Action=<op>` dispatch,
//! responses are `text/xml`.
//!
//! SES v2 uses `restJson1` with path-based routing under `/v2/email/`.

pub mod body;
pub mod dispatch;
pub mod request;
pub mod response;
pub mod router;
pub mod service;
pub mod v2;
