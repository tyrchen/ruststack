//! SNS HTTP service layer for RustStack (awsQuery protocol).
//!
//! SNS is the only AWS service using purely the `awsQuery` protocol.
//! Requests are `application/x-www-form-urlencoded` with `Action=<op>` dispatch.
//! Responses are `text/xml`.

pub mod body;
pub mod dispatch;
pub mod request;
pub mod response;
pub mod router;
pub mod service;
