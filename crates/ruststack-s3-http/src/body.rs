//! S3 response body types supporting buffered and empty modes.
//!
//! This module provides [`S3ResponseBody`], the HTTP response body type used throughout
//! the S3 HTTP service. It supports two modes:
//!
//! - **Buffered**: For small responses such as XML payloads, error bodies, and raw bytes.
//! - **Empty**: For responses with no body content (e.g., 204 No Content, HEAD responses).
//!
//! Streaming support for large objects (e.g., `GetObject`) can be added in the future
//! by extending this enum with a streaming variant.

use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use http_body_util::Full;

/// S3 response body supporting buffered and empty modes.
///
/// Implements [`http_body::Body`] so it can be used directly with hyper responses.
#[derive(Debug, Default)]
pub enum S3ResponseBody {
    /// Buffered body for small responses: XML payloads, error bodies, raw bytes.
    Buffered(Full<Bytes>),
    /// Empty body for 204 responses, DELETE confirmations, HEAD responses, etc.
    #[default]
    Empty,
}

impl S3ResponseBody {
    /// Create a buffered body from bytes.
    #[must_use]
    pub fn from_bytes(data: impl Into<Bytes>) -> Self {
        Self::Buffered(Full::new(data.into()))
    }

    /// Create an empty body.
    #[must_use]
    pub fn empty() -> Self {
        Self::Empty
    }

    /// Create a buffered body from a UTF-8 string.
    #[must_use]
    pub fn from_string(s: impl Into<String>) -> Self {
        Self::Buffered(Full::new(Bytes::from(s.into())))
    }

    /// Create a buffered body from an XML byte vector.
    #[must_use]
    pub fn from_xml(xml: Vec<u8>) -> Self {
        Self::Buffered(Full::new(Bytes::from(xml)))
    }
}

impl http_body::Body for S3ResponseBody {
    type Data = Bytes;
    type Error = std::io::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        match self.get_mut() {
            Self::Buffered(full) => Pin::new(full)
                .poll_frame(cx)
                .map_err(|never| match never {}),
            Self::Empty => Poll::Ready(None),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            Self::Buffered(full) => full.is_end_stream(),
            Self::Empty => true,
        }
    }

    fn size_hint(&self) -> http_body::SizeHint {
        match self {
            Self::Buffered(full) => full.size_hint(),
            Self::Empty => http_body::SizeHint::with_exact(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use http_body::Body;

    use super::*;

    #[test]
    fn test_should_report_empty_body_as_end_of_stream() {
        let body = S3ResponseBody::empty();
        assert!(body.is_end_stream());
    }

    #[test]
    fn test_should_have_zero_size_for_empty_body() {
        let body = S3ResponseBody::empty();
        let hint = body.size_hint();
        assert_eq!(hint.exact(), Some(0));
    }

    #[test]
    fn test_should_create_buffered_body_from_bytes() {
        let body = S3ResponseBody::from_bytes(Bytes::from("hello"));
        assert!(!body.is_end_stream());
        let hint = body.size_hint();
        assert_eq!(hint.exact(), Some(5));
    }

    #[test]
    fn test_should_create_buffered_body_from_string() {
        let body = S3ResponseBody::from_string("hello world");
        assert!(!body.is_end_stream());
        let hint = body.size_hint();
        assert_eq!(hint.exact(), Some(11));
    }

    #[test]
    fn test_should_create_buffered_body_from_xml() {
        let xml = b"<Root><Key>value</Key></Root>".to_vec();
        let body = S3ResponseBody::from_xml(xml);
        assert!(!body.is_end_stream());
    }

    #[test]
    fn test_should_default_to_empty() {
        let body = S3ResponseBody::default();
        assert!(body.is_end_stream());
    }
}
