//! Validation for S3 requests.
//!
//! Provides validation functions for bucket names, object keys, tags, and
//! user-defined metadata following the rules defined in the
//! [Amazon S3 documentation](https://docs.aws.amazon.com/AmazonS3/latest/userguide/bucketnamingrules.html).

use std::collections::HashMap;
use std::hash::BuildHasher;
use std::net::Ipv4Addr;

use base64::Engine;
use md5::{Digest, Md5};

use crate::error::S3ServiceError;

/// Maximum number of tags allowed on a single S3 object or bucket.
const MAX_TAGS: usize = 10;

/// Maximum length of a tag key in characters.
const MAX_TAG_KEY_LEN: usize = 128;

/// Maximum length of a tag value in characters.
const MAX_TAG_VALUE_LEN: usize = 256;

/// Maximum total size (in bytes) of all user-defined metadata keys and values.
const MAX_METADATA_SIZE: usize = 2048;

/// Maximum object key length in bytes.
const MAX_KEY_BYTES: usize = 1024;

/// Minimum bucket name length.
const MIN_BUCKET_NAME_LEN: usize = 3;

/// Maximum bucket name length.
const MAX_BUCKET_NAME_LEN: usize = 63;

/// Validate an S3 bucket name.
///
/// Rules (per AWS documentation):
/// - 3-63 characters long
/// - Only lowercase letters, numbers, hyphens, and dots
/// - Must start and end with a letter or number
/// - No consecutive dots (`..`)
/// - Not formatted as an IPv4 address (e.g. `192.168.0.1`)
/// - Must not start with `xn--`
/// - Must not end with `-s3alias`
/// - Must not start with `sthree-`
///
/// # Errors
///
/// Returns [`S3ServiceError::InvalidBucketName`] if any rule is violated.
///
/// # Examples
///
/// ```
/// use ruststack_s3_core::validation::validate_bucket_name;
///
/// assert!(validate_bucket_name("my-valid-bucket").is_ok());
/// assert!(validate_bucket_name("AB").is_err());
/// ```
pub fn validate_bucket_name(name: &str) -> Result<(), S3ServiceError> {
    let len = name.len();

    if !(MIN_BUCKET_NAME_LEN..=MAX_BUCKET_NAME_LEN).contains(&len) {
        return Err(S3ServiceError::InvalidBucketName {
            name: name.to_owned(),
            reason: format!(
                "Bucket name must be between {MIN_BUCKET_NAME_LEN} and {MAX_BUCKET_NAME_LEN} characters long"
            ),
        });
    }

    if !name
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.')
    {
        return Err(S3ServiceError::InvalidBucketName {
            name: name.to_owned(),
            reason: "Bucket name must only contain lowercase letters, numbers, hyphens, and dots"
                .to_owned(),
        });
    }

    let first = name.as_bytes()[0];
    let last = name.as_bytes()[len - 1];
    if !(first.is_ascii_lowercase() || first.is_ascii_digit())
        || !(last.is_ascii_lowercase() || last.is_ascii_digit())
    {
        return Err(S3ServiceError::InvalidBucketName {
            name: name.to_owned(),
            reason: "Bucket name must start and end with a letter or number".to_owned(),
        });
    }

    if name.contains("..") {
        return Err(S3ServiceError::InvalidBucketName {
            name: name.to_owned(),
            reason: "Bucket name must not contain consecutive dots".to_owned(),
        });
    }

    if name.parse::<Ipv4Addr>().is_ok() {
        return Err(S3ServiceError::InvalidBucketName {
            name: name.to_owned(),
            reason: "Bucket name must not be formatted as an IP address".to_owned(),
        });
    }

    if name.starts_with("xn--") {
        return Err(S3ServiceError::InvalidBucketName {
            name: name.to_owned(),
            reason: "Bucket name must not start with 'xn--'".to_owned(),
        });
    }

    if name.ends_with("-s3alias") {
        return Err(S3ServiceError::InvalidBucketName {
            name: name.to_owned(),
            reason: "Bucket name must not end with '-s3alias'".to_owned(),
        });
    }

    if name.starts_with("sthree-") {
        return Err(S3ServiceError::InvalidBucketName {
            name: name.to_owned(),
            reason: "Bucket name must not start with 'sthree-'".to_owned(),
        });
    }

    Ok(())
}

/// Validate an S3 object key.
///
/// Rules:
/// - 1-1024 bytes in length
/// - Must be valid UTF-8 (enforced by the `&str` type)
///
/// # Errors
///
/// Returns [`S3ServiceError::InvalidArgument`] if the key is empty or exceeds
/// 1024 bytes, or [`S3ServiceError::KeyTooLong`] if the key is too long.
///
/// # Examples
///
/// ```
/// use ruststack_s3_core::validation::validate_object_key;
///
/// assert!(validate_object_key("photos/2024/image.jpg").is_ok());
/// assert!(validate_object_key("").is_err());
/// ```
pub fn validate_object_key(key: &str) -> Result<(), S3ServiceError> {
    if key.is_empty() {
        return Err(S3ServiceError::InvalidArgument {
            message: "Object key must not be empty".to_owned(),
        });
    }

    if key.len() > MAX_KEY_BYTES {
        return Err(S3ServiceError::KeyTooLong);
    }

    Ok(())
}

/// Validate a tag key.
///
/// Rules:
/// - 1-128 characters in length
///
/// # Errors
///
/// Returns [`S3ServiceError::InvalidTag`] if the key is empty or too long.
///
/// # Examples
///
/// ```
/// use ruststack_s3_core::validation::validate_tag_key;
///
/// assert!(validate_tag_key("environment").is_ok());
/// assert!(validate_tag_key("").is_err());
/// ```
pub fn validate_tag_key(key: &str) -> Result<(), S3ServiceError> {
    if key.is_empty() {
        return Err(S3ServiceError::InvalidTag {
            message: "Tag key must not be empty".to_owned(),
        });
    }
    if key.chars().count() > MAX_TAG_KEY_LEN {
        return Err(S3ServiceError::InvalidTag {
            message: format!(
                "Tag key must not exceed {MAX_TAG_KEY_LEN} characters, got {}",
                key.chars().count()
            ),
        });
    }
    Ok(())
}

/// Validate a tag value.
///
/// Rules:
/// - 0-256 characters in length (empty values are allowed)
///
/// # Errors
///
/// Returns [`S3ServiceError::InvalidTag`] if the value exceeds 256 characters.
///
/// # Examples
///
/// ```
/// use ruststack_s3_core::validation::validate_tag_value;
///
/// assert!(validate_tag_value("production").is_ok());
/// assert!(validate_tag_value("").is_ok());
/// ```
pub fn validate_tag_value(value: &str) -> Result<(), S3ServiceError> {
    if value.chars().count() > MAX_TAG_VALUE_LEN {
        return Err(S3ServiceError::InvalidTag {
            message: format!(
                "Tag value must not exceed {MAX_TAG_VALUE_LEN} characters, got {}",
                value.chars().count()
            ),
        });
    }
    Ok(())
}

/// Validate a set of tags.
///
/// Rules:
/// - Maximum of 10 tags
/// - Each key must be 1-128 characters
/// - Each value must be 0-256 characters
///
/// # Errors
///
/// Returns [`S3ServiceError::InvalidTag`] if any rule is violated.
///
/// # Examples
///
/// ```
/// use ruststack_s3_core::validation::validate_tags;
///
/// let tags = vec![
///     ("env".to_owned(), "prod".to_owned()),
///     ("team".to_owned(), "backend".to_owned()),
/// ];
/// assert!(validate_tags(&tags).is_ok());
/// ```
pub fn validate_tags(tags: &[(String, String)]) -> Result<(), S3ServiceError> {
    if tags.len() > MAX_TAGS {
        return Err(S3ServiceError::InvalidTag {
            message: format!(
                "Object tags cannot be greater than {MAX_TAGS}, got {}",
                tags.len()
            ),
        });
    }

    for (key, value) in tags {
        validate_tag_key(key)?;
        validate_tag_value(value)?;
    }

    Ok(())
}

/// Validate user-defined metadata.
///
/// Rules:
/// - Total size of all keys plus all values must not exceed 2 KB (2048 bytes)
///
/// # Errors
///
/// Returns [`S3ServiceError::InvalidArgument`] if the total metadata size
/// exceeds the limit.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use ruststack_s3_core::validation::validate_metadata;
///
/// let mut meta = HashMap::new();
/// meta.insert("color".to_owned(), "blue".to_owned());
/// assert!(validate_metadata(&meta).is_ok());
/// ```
pub fn validate_metadata<S: BuildHasher>(
    metadata: &HashMap<String, String, S>,
) -> Result<(), S3ServiceError> {
    let total_size: usize = metadata.iter().map(|(k, v)| k.len() + v.len()).sum();

    if total_size > MAX_METADATA_SIZE {
        return Err(S3ServiceError::InvalidArgument {
            message: format!(
                "User-defined metadata must not exceed {MAX_METADATA_SIZE} bytes, got {total_size}"
            ),
        });
    }

    Ok(())
}

/// Validate the `Content-MD5` header against the request body.
///
/// If the header is present, its value must be a valid Base64-encoded MD5
/// digest that matches the body. If the header is absent, validation
/// succeeds (the header is optional).
///
/// # Errors
///
/// Returns [`S3ServiceError::InvalidDigest`] if the header value is not
/// valid Base64, or [`S3ServiceError::BadDigest`] if the decoded digest
/// does not match the body.
///
/// # Examples
///
/// ```
/// use ruststack_s3_core::validation::validate_content_md5;
///
/// // No header → always OK
/// assert!(validate_content_md5(None, b"hello").is_ok());
/// ```
pub fn validate_content_md5(content_md5: Option<&str>, body: &[u8]) -> Result<(), S3ServiceError> {
    let Some(expected_b64) = content_md5 else {
        return Ok(());
    };

    let expected_bytes = base64::engine::general_purpose::STANDARD
        .decode(expected_b64)
        .map_err(|_| S3ServiceError::InvalidDigest)?;

    let actual = Md5::digest(body);
    if actual.as_slice() != expected_bytes {
        return Err(S3ServiceError::BadDigest);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Bucket name validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_accept_valid_bucket_names() {
        let long_name = "a".repeat(63);
        let valid = [
            "my-bucket",
            "abc",
            "a-b-c",
            "bucket.with.dots",
            "123bucket",
            "bucket123",
            long_name.as_str(),
        ];
        for name in valid {
            assert!(validate_bucket_name(name).is_ok(), "expected valid: {name}");
        }
    }

    #[test]
    fn test_should_reject_short_bucket_name() {
        assert!(validate_bucket_name("ab").is_err());
        assert!(validate_bucket_name("a").is_err());
        assert!(validate_bucket_name("").is_err());
    }

    #[test]
    fn test_should_reject_long_bucket_name() {
        let name = "a".repeat(64);
        assert!(validate_bucket_name(&name).is_err());
    }

    #[test]
    fn test_should_reject_uppercase_bucket_name() {
        assert!(validate_bucket_name("MyBucket").is_err());
    }

    #[test]
    fn test_should_reject_bucket_starting_with_hyphen() {
        assert!(validate_bucket_name("-bucket").is_err());
    }

    #[test]
    fn test_should_reject_bucket_ending_with_hyphen() {
        assert!(validate_bucket_name("bucket-").is_err());
    }

    #[test]
    fn test_should_reject_consecutive_dots_in_bucket_name() {
        assert!(validate_bucket_name("my..bucket").is_err());
    }

    #[test]
    fn test_should_reject_ip_address_bucket_name() {
        assert!(validate_bucket_name("192.168.1.1").is_err());
    }

    #[test]
    fn test_should_reject_xn_prefix_bucket_name() {
        assert!(validate_bucket_name("xn--example").is_err());
    }

    #[test]
    fn test_should_reject_s3alias_suffix_bucket_name() {
        assert!(validate_bucket_name("mybucket-s3alias").is_err());
    }

    #[test]
    fn test_should_reject_sthree_prefix_bucket_name() {
        assert!(validate_bucket_name("sthree-bucket").is_err());
    }

    // -----------------------------------------------------------------------
    // Object key validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_accept_valid_object_keys() {
        assert!(validate_object_key("a").is_ok());
        assert!(validate_object_key("photos/2024/image.jpg").is_ok());
        assert!(validate_object_key(&"k".repeat(1024)).is_ok());
    }

    #[test]
    fn test_should_reject_empty_object_key() {
        assert!(validate_object_key("").is_err());
    }

    #[test]
    fn test_should_reject_too_long_object_key() {
        let key = "k".repeat(1025);
        assert!(validate_object_key(&key).is_err());
    }

    // -----------------------------------------------------------------------
    // Tag validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_accept_valid_tag_key() {
        assert!(validate_tag_key("environment").is_ok());
        assert!(validate_tag_key(&"k".repeat(128)).is_ok());
    }

    #[test]
    fn test_should_reject_empty_tag_key() {
        assert!(validate_tag_key("").is_err());
    }

    #[test]
    fn test_should_reject_too_long_tag_key() {
        assert!(validate_tag_key(&"k".repeat(129)).is_err());
    }

    #[test]
    fn test_should_accept_valid_tag_value() {
        assert!(validate_tag_value("").is_ok());
        assert!(validate_tag_value("production").is_ok());
        assert!(validate_tag_value(&"v".repeat(256)).is_ok());
    }

    #[test]
    fn test_should_reject_too_long_tag_value() {
        assert!(validate_tag_value(&"v".repeat(257)).is_err());
    }

    #[test]
    fn test_should_accept_valid_tag_set() {
        let tags: Vec<(String, String)> = (0..10)
            .map(|i| (format!("key{i}"), format!("val{i}")))
            .collect();
        assert!(validate_tags(&tags).is_ok());
    }

    #[test]
    fn test_should_reject_too_many_tags() {
        let tags: Vec<(String, String)> = (0..11)
            .map(|i| (format!("key{i}"), format!("val{i}")))
            .collect();
        assert!(validate_tags(&tags).is_err());
    }

    #[test]
    fn test_should_reject_tags_with_invalid_key() {
        let tags = vec![(String::new(), "value".to_owned())];
        assert!(validate_tags(&tags).is_err());
    }

    #[test]
    fn test_should_reject_tags_with_invalid_value() {
        let tags = vec![("key".to_owned(), "v".repeat(257))];
        assert!(validate_tags(&tags).is_err());
    }

    // -----------------------------------------------------------------------
    // Metadata validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_accept_valid_metadata() {
        let mut meta = HashMap::new();
        meta.insert("color".to_owned(), "blue".to_owned());
        assert!(validate_metadata(&meta).is_ok());
    }

    #[test]
    fn test_should_accept_empty_metadata() {
        let meta = HashMap::new();
        assert!(validate_metadata(&meta).is_ok());
    }

    #[test]
    fn test_should_reject_oversized_metadata() {
        let mut meta = HashMap::new();
        // Single entry that exceeds 2 KB
        meta.insert("key".to_owned(), "v".repeat(2048));
        assert!(validate_metadata(&meta).is_err());
    }

    #[test]
    fn test_should_accept_metadata_at_limit() {
        let mut meta = HashMap::new();
        // key (3 bytes) + value (2045 bytes) = 2048
        meta.insert("key".to_owned(), "v".repeat(2045));
        assert!(validate_metadata(&meta).is_ok());
    }

    // -----------------------------------------------------------------------
    // Content-MD5 validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_accept_absent_content_md5() {
        assert!(validate_content_md5(None, b"any body").is_ok());
    }

    #[test]
    fn test_should_accept_correct_content_md5() {
        let body = b"hello world";
        let digest = base64::engine::general_purpose::STANDARD.encode(Md5::digest(body));
        assert!(validate_content_md5(Some(&digest), body).is_ok());
    }

    #[test]
    fn test_should_reject_wrong_content_md5() {
        let body = b"hello world";
        let wrong = base64::engine::general_purpose::STANDARD.encode(Md5::digest(b"wrong"));
        assert!(matches!(
            validate_content_md5(Some(&wrong), body),
            Err(S3ServiceError::BadDigest)
        ));
    }

    #[test]
    fn test_should_reject_invalid_base64_content_md5() {
        assert!(matches!(
            validate_content_md5(Some("not-valid-base64!!!"), b"body"),
            Err(S3ServiceError::InvalidDigest)
        ));
    }
}
