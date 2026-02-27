//! Core types, configuration, and state management for RustStack.
//!
//! This crate provides the foundational building blocks shared across all
//! RustStack service implementations, including multi-account/multi-region
//! state management, configuration, and common AWS type definitions.

mod config;
mod error;
mod state;
mod types;

pub use config::RustStackConfig;
pub use error::{RustStackError, RustStackResult};
pub use state::AccountRegionStore;
pub use types::{AccountId, AwsRegion};
