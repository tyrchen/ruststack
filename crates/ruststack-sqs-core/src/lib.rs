//! SQS business logic and queue engine for RustStack.
//!
//! This crate implements the core SQS functionality using an actor-per-queue
//! concurrency model. Each queue runs as an independent actor that owns its
//! message state and communicates via `tokio::sync::mpsc` channels.

pub mod config;
pub mod handler;
pub mod message;
pub mod provider;
pub mod queue;
