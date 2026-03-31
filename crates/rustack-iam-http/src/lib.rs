//! IAM HTTP service layer for Rustack (awsQuery protocol).
//!
//! IAM uses the `awsQuery` protocol (same as SNS). Requests are
//! `application/x-www-form-urlencoded` with `Action=<op>` dispatch.
//! Responses are `text/xml`.

pub mod body;
pub mod dispatch;
pub mod request;
pub mod response;
pub mod router;
pub mod service;
