//! DynamoDB Streams core business logic for Rustack.
#![allow(missing_docs, clippy::doc_markdown, clippy::module_name_repetitions)]

pub mod config;
pub mod emitter;
pub mod handler;
pub mod iterator;
pub mod provider;
pub mod storage;
