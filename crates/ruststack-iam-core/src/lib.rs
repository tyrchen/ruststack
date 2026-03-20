//! IAM business logic for `RustStack`.
//!
//! Implements user, role, group, policy, instance profile, and access key
//! management with in-memory state. Covers all four implementation phases
//! (~60 operations).

pub mod arn;
pub mod config;
pub mod handler;
pub mod id_gen;
pub mod provider;
pub mod store;
pub mod types;
pub mod validation;
