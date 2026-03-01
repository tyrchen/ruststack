//! DynamoDB model types for RustStack.
//!
//! This crate provides all DynamoDB API types needed for the RustStack DynamoDB
//! implementation. Unlike the S3 model crate which is auto-generated from Smithy,
//! these types are hand-written since DynamoDB's JSON protocol makes serde derives
//! trivial.
// "DynamoDB" appears in virtually every doc comment in this crate.
#![allow(clippy::doc_markdown)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::module_name_repetitions)]
#![allow(missing_docs)]

pub mod attribute_value;
pub mod error;
pub mod input;
pub mod operations;
pub mod output;
pub mod types;

pub use attribute_value::AttributeValue;
pub use error::{DynamoDBError, DynamoDBErrorCode};
pub use operations::DynamoDBOperation;
