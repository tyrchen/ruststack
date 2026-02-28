//! AWS chunked transfer encoding decoder.
//!
//! When S3 clients (e.g. minio-go) send PutObject with `Content-Encoding:
//! aws-chunked`, the body uses a proprietary chunked format:
//!
//! ```text
//! <hex-size>;chunk-signature=<sig>\r\n
//! <data>\r\n
//! 0;chunk-signature=<sig>\r\n
//! \r\n
//! ```
//!
//! This module detects and decodes that framing so the server stores the raw
//! object data rather than the chunk envelope.

use bytes::{Bytes, BytesMut};
use http::header::HeaderMap;
use ruststack_s3_model::error::{S3Error, S3ErrorCode};

/// Return `true` if the request uses AWS chunked transfer encoding.
///
/// Detection checks:
/// - `Content-Encoding` header contains `aws-chunked`, OR
/// - `x-amz-content-sha256` starts with `STREAMING-`
pub fn is_aws_chunked(parts: &http::request::Parts) -> bool {
    if let Some(ce) = parts.headers.get(http::header::CONTENT_ENCODING) {
        if let Ok(s) = ce.to_str() {
            if s.contains("aws-chunked") {
                return true;
            }
        }
    }

    if let Some(sha) = parts.headers.get("x-amz-content-sha256") {
        if let Ok(s) = sha.to_str() {
            if s.starts_with("STREAMING-") {
                return true;
            }
        }
    }

    false
}

/// Decode an AWS-chunked body into the raw payload bytes.
///
/// # Errors
///
/// Returns an error if the chunked framing is malformed (missing size line,
/// invalid hex size, or truncated data).
pub fn decode_aws_chunked(body: &[u8]) -> Result<Bytes, S3Error> {
    let mut output = BytesMut::new();
    let mut pos = 0;

    loop {
        // Find the end of the size line (\r\n).
        let line_end = find_crlf(body, pos).ok_or_else(|| {
            S3Error::with_message(
                S3ErrorCode::InvalidArgument,
                "Malformed aws-chunked body: missing chunk size line",
            )
        })?;

        let size_line = &body[pos..line_end];

        // The size line format is: <hex-size>[;chunk-signature=<sig>][;other-ext]
        // Extract the hex size (everything before the first `;`).
        let hex_part = if let Some(semi) = size_line.iter().position(|&b| b == b';') {
            &size_line[..semi]
        } else {
            size_line
        };

        let hex_str = std::str::from_utf8(hex_part).map_err(|_| {
            S3Error::with_message(
                S3ErrorCode::InvalidArgument,
                "Malformed aws-chunked body: invalid chunk size encoding",
            )
        })?;

        let chunk_size = usize::from_str_radix(hex_str.trim(), 16).map_err(|_| {
            S3Error::with_message(
                S3ErrorCode::InvalidArgument,
                format!("Malformed aws-chunked body: invalid chunk size '{hex_str}'"),
            )
        })?;

        // Skip past the size line CRLF.
        pos = line_end + 2;

        if chunk_size == 0 {
            // Terminal chunk — we're done.
            break;
        }

        // Read exactly `chunk_size` bytes of data.
        if pos + chunk_size > body.len() {
            return Err(S3Error::with_message(
                S3ErrorCode::InvalidArgument,
                "Malformed aws-chunked body: chunk data truncated",
            ));
        }

        output.extend_from_slice(&body[pos..pos + chunk_size]);
        pos += chunk_size;

        // Expect trailing CRLF after the chunk data.
        if pos + 2 > body.len() || body[pos] != b'\r' || body[pos + 1] != b'\n' {
            return Err(S3Error::with_message(
                S3ErrorCode::InvalidArgument,
                "Malformed aws-chunked body: missing CRLF after chunk data",
            ));
        }
        pos += 2;
    }

    Ok(output.freeze())
}

/// Remove `aws-chunked` from the `Content-Encoding` header.
///
/// If the header becomes empty after removal, the entire header is deleted.
pub fn strip_aws_chunked_encoding(headers: &mut HeaderMap) {
    let Some(ce) = headers.get(http::header::CONTENT_ENCODING) else {
        return;
    };

    let Ok(value) = ce.to_str() else {
        return;
    };

    let remaining: Vec<&str> = value
        .split(',')
        .map(str::trim)
        .filter(|&v| !v.eq_ignore_ascii_case("aws-chunked"))
        .collect();

    if remaining.is_empty() {
        headers.remove(http::header::CONTENT_ENCODING);
    } else if let Ok(new_val) = http::header::HeaderValue::from_str(&remaining.join(", ")) {
        headers.insert(http::header::CONTENT_ENCODING, new_val);
    }
}

/// Find the position of the next `\r\n` starting from `start`.
fn find_crlf(data: &[u8], start: usize) -> Option<usize> {
    if data.len() < start + 2 {
        return None;
    }
    data[start..]
        .windows(2)
        .position(|w| w == b"\r\n")
        .map(|p| start + p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_parts(content_encoding: Option<&str>, sha256: Option<&str>) -> http::request::Parts {
        let mut builder = http::Request::builder().method("PUT").uri("/test");
        if let Some(ce) = content_encoding {
            builder = builder.header("content-encoding", ce);
        }
        if let Some(sha) = sha256 {
            builder = builder.header("x-amz-content-sha256", sha);
        }
        let (parts, ()) = builder.body(()).expect("valid request").into_parts();
        parts
    }

    #[test]
    fn test_should_detect_aws_chunked_content_encoding() {
        let parts = make_parts(Some("aws-chunked"), None);
        assert!(is_aws_chunked(&parts));
    }

    #[test]
    fn test_should_detect_streaming_sha256() {
        let parts = make_parts(None, Some("STREAMING-AWS4-HMAC-SHA256-PAYLOAD"));
        assert!(is_aws_chunked(&parts));
    }

    #[test]
    fn test_should_not_detect_plain_request() {
        let parts = make_parts(None, Some("UNSIGNED-PAYLOAD"));
        assert!(!is_aws_chunked(&parts));
    }

    #[test]
    fn test_should_not_detect_no_headers() {
        let parts = make_parts(None, None);
        assert!(!is_aws_chunked(&parts));
    }

    #[test]
    fn test_should_decode_single_chunk() {
        let body = b"5;chunk-signature=abc123\r\nhello\r\n0;chunk-signature=def456\r\n\r\n";
        let result = decode_aws_chunked(body).expect("should decode");
        assert_eq!(result.as_ref(), b"hello");
    }

    #[test]
    fn test_should_decode_multiple_chunks() {
        let body =
            b"5;chunk-signature=aaa\r\nhello\r\n6;chunk-signature=bbb\r\n world\r\n0;chunk-signature=ccc\r\n\r\n";
        let result = decode_aws_chunked(body).expect("should decode");
        assert_eq!(result.as_ref(), b"hello world");
    }

    #[test]
    fn test_should_decode_empty_body() {
        let body = b"0;chunk-signature=abc\r\n\r\n";
        let result = decode_aws_chunked(body).expect("should decode");
        assert!(result.is_empty());
    }

    #[test]
    fn test_should_reject_malformed_no_crlf() {
        let body = b"5;chunk-signature=abc";
        assert!(decode_aws_chunked(body).is_err());
    }

    #[test]
    fn test_should_reject_truncated_data() {
        let body = b"10;chunk-signature=abc\r\nshort\r\n";
        assert!(decode_aws_chunked(body).is_err());
    }

    #[test]
    fn test_should_strip_aws_chunked_encoding_only() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::CONTENT_ENCODING,
            "aws-chunked".parse().unwrap(),
        );
        strip_aws_chunked_encoding(&mut headers);
        assert!(headers.get(http::header::CONTENT_ENCODING).is_none());
    }

    #[test]
    fn test_should_strip_aws_chunked_keep_other() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::CONTENT_ENCODING,
            "gzip, aws-chunked".parse().unwrap(),
        );
        strip_aws_chunked_encoding(&mut headers);
        assert_eq!(
            headers
                .get(http::header::CONTENT_ENCODING)
                .unwrap()
                .to_str()
                .unwrap(),
            "gzip"
        );
    }

    #[test]
    fn test_should_decode_chunk_without_signature_extension() {
        let body = b"3\r\nabc\r\n0\r\n\r\n";
        let result = decode_aws_chunked(body).expect("should decode");
        assert_eq!(result.as_ref(), b"abc");
    }
}
