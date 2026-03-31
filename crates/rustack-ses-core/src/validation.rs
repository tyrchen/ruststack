//! Validation utilities for SES operations.
//!
//! Provides tag validation and basic email address format checks.

use rustack_ses_model::error::{SesError, SesErrorCode};

/// Maximum length for tag names and values.
const MAX_TAG_LENGTH: usize = 255;

/// Validate a message tag name.
///
/// Tag names must be:
/// - Non-empty
/// - At most 255 characters
/// - Composed of `[A-Za-z0-9_-]` characters (with `ses:` prefix exception)
///
/// # Errors
///
/// Returns `SesError` with `InvalidParameterValue` if the tag name is invalid.
pub fn validate_tag_name(name: &str) -> Result<(), SesError> {
    if name.is_empty() {
        return Err(SesError::with_message(
            SesErrorCode::InvalidParameterValue,
            "Tag name must not be empty.",
        ));
    }
    if name.len() > MAX_TAG_LENGTH {
        return Err(SesError::with_message(
            SesErrorCode::InvalidParameterValue,
            format!(
                "Tag name must be at most {MAX_TAG_LENGTH} characters, got {}.",
                name.len()
            ),
        ));
    }
    // Allow ses: prefix (AWS reserved tags)
    let check_part = name.strip_prefix("ses:").unwrap_or(name);
    if !check_part
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(SesError::with_message(
            SesErrorCode::InvalidParameterValue,
            format!("Tag name contains invalid characters: {name}"),
        ));
    }
    Ok(())
}

/// Validate a message tag value.
///
/// Tag values must be:
/// - Non-empty
/// - At most 255 characters
/// - Composed of `[A-Za-z0-9_\-.@]` characters
///
/// # Errors
///
/// Returns `SesError` with `InvalidParameterValue` if the tag value is invalid.
pub fn validate_tag_value(value: &str) -> Result<(), SesError> {
    if value.is_empty() {
        return Err(SesError::with_message(
            SesErrorCode::InvalidParameterValue,
            "Tag value must not be empty.",
        ));
    }
    if value.len() > MAX_TAG_LENGTH {
        return Err(SesError::with_message(
            SesErrorCode::InvalidParameterValue,
            format!(
                "Tag value must be at most {MAX_TAG_LENGTH} characters, got {}.",
                value.len()
            ),
        ));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '@')
    {
        return Err(SesError::with_message(
            SesErrorCode::InvalidParameterValue,
            format!("Tag value contains invalid characters: {value}"),
        ));
    }
    Ok(())
}

/// Validate a list of message tags (name/value pairs).
///
/// # Errors
///
/// Returns the first validation error encountered.
pub fn validate_tags(tags: &[(String, String)]) -> Result<(), SesError> {
    for (name, value) in tags {
        validate_tag_name(name)?;
        validate_tag_value(value)?;
    }
    Ok(())
}

/// Basic email address validation.
///
/// Checks that the address contains an `@` symbol. This is intentionally
/// minimal -- full RFC 5322 validation is not required for local development.
///
/// # Errors
///
/// Returns `SesError` with `InvalidParameterValue` if the email is invalid.
pub fn validate_email_address(email: &str) -> Result<(), SesError> {
    if !email.contains('@') {
        return Err(SesError::with_message(
            SesErrorCode::InvalidParameterValue,
            format!("Invalid email address: {email}"),
        ));
    }
    Ok(())
}

/// Extract the domain from an email address.
///
/// Returns `None` if the email does not contain `@`.
#[must_use]
pub fn extract_domain(email: &str) -> Option<&str> {
    email.split('@').nth(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_accept_valid_tag_name() {
        assert!(validate_tag_name("campaign").is_ok());
        assert!(validate_tag_name("my-tag").is_ok());
        assert!(validate_tag_name("my_tag").is_ok());
        assert!(validate_tag_name("Tag123").is_ok());
    }

    #[test]
    fn test_should_accept_ses_prefix_tag_name() {
        assert!(validate_tag_name("ses:campaign").is_ok());
        assert!(validate_tag_name("ses:feedback-id").is_ok());
    }

    #[test]
    fn test_should_reject_empty_tag_name() {
        assert!(validate_tag_name("").is_err());
    }

    #[test]
    fn test_should_reject_too_long_tag_name() {
        let long_name = "a".repeat(256);
        assert!(validate_tag_name(&long_name).is_err());
    }

    #[test]
    fn test_should_reject_invalid_chars_in_tag_name() {
        assert!(validate_tag_name("tag name").is_err());
        assert!(validate_tag_name("tag.name").is_err());
        assert!(validate_tag_name("tag@name").is_err());
    }

    #[test]
    fn test_should_accept_valid_tag_value() {
        assert!(validate_tag_value("welcome").is_ok());
        assert!(validate_tag_value("test-value").is_ok());
        assert!(validate_tag_value("user@example.com").is_ok());
        assert!(validate_tag_value("value_123").is_ok());
        assert!(validate_tag_value("v1.2.3").is_ok());
    }

    #[test]
    fn test_should_reject_empty_tag_value() {
        assert!(validate_tag_value("").is_err());
    }

    #[test]
    fn test_should_reject_too_long_tag_value() {
        let long_value = "a".repeat(256);
        assert!(validate_tag_value(&long_value).is_err());
    }

    #[test]
    fn test_should_reject_invalid_chars_in_tag_value() {
        assert!(validate_tag_value("value with spaces").is_err());
        assert!(validate_tag_value("value<html>").is_err());
    }

    #[test]
    fn test_should_validate_tag_pairs() {
        let tags = vec![
            ("campaign".to_owned(), "welcome".to_owned()),
            ("source".to_owned(), "test".to_owned()),
        ];
        assert!(validate_tags(&tags).is_ok());
    }

    #[test]
    fn test_should_reject_invalid_tag_in_list() {
        let tags = vec![
            ("campaign".to_owned(), "welcome".to_owned()),
            (String::new(), "value".to_owned()),
        ];
        assert!(validate_tags(&tags).is_err());
    }

    #[test]
    fn test_should_accept_valid_email() {
        assert!(validate_email_address("user@example.com").is_ok());
        assert!(validate_email_address("a@b").is_ok());
    }

    #[test]
    fn test_should_reject_email_without_at() {
        assert!(validate_email_address("userexample.com").is_err());
        assert!(validate_email_address("").is_err());
    }

    #[test]
    fn test_should_extract_domain() {
        assert_eq!(extract_domain("user@example.com"), Some("example.com"));
        assert_eq!(extract_domain("noat"), None);
    }
}
