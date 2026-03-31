//! SES business logic for `Rustack`.
//!
//! Implements email sending (capture), identity management, template management,
//! configuration sets, receipt rules, and the email retrospection endpoint.
//! No actual email delivery -- all emails are captured in memory for test
//! inspection via `/_aws/ses`.

pub mod config;
pub mod config_set;
pub mod handler;
pub mod identity;
pub mod provider;
pub mod receipt_rule;
pub mod retrospection;
pub mod statistics;
pub mod template;
pub mod validation;
