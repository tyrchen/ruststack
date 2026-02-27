//! S3 XML serialization/deserialization for `RustStack`.
//!
//! This crate provides the XML layer for the S3 REST protocol, handling conversion
//! between S3 model types and the XML wire format. S3 uses the RestXml protocol with
//! `noErrorWrapping: true`.
//!
//! # Key components
//!
//! - [`S3Serialize`] trait and [`to_xml`] function for serializing structs to XML response bodies
//! - [`S3Deserialize`] trait and [`from_xml`] function for parsing XML request bodies into structs
//! - [`error_to_xml`] for formatting S3 error responses as XML
//!
//! # S3 XML conventions
//!
//! - Namespace: `http://s3.amazonaws.com/doc/2006-03-01/`
//! - Booleans: lowercase `true`/`false`
//! - Timestamps: ISO 8601 format (`2006-02-03T16:45:09.000Z`)
//! - XML declaration: `<?xml version="1.0" encoding="UTF-8"?>`

pub mod deserialize;
pub mod error;
pub mod serialize;

pub use deserialize::{S3Deserialize, from_xml};
pub use error::{XmlError, error_to_xml};
pub use serialize::{S3_NAMESPACE, S3Serialize, to_xml};
