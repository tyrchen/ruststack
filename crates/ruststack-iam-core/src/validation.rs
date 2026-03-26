//! Input validation for IAM operations.
//!
//! Validates entity names, paths, policy documents, and session durations
//! against AWS IAM constraints.

use ruststack_iam_model::error::IamError;

/// Validate an IAM entity name (user, role, group, policy, instance profile).
///
/// Entity names must be 1 to `max_len` characters and may only contain
/// alphanumeric characters plus `+=,.@_-`.
///
/// # Errors
///
/// Returns [`IamError`] if the name is empty, too long, or contains
/// invalid characters.
pub fn validate_entity_name(name: &str, max_len: usize) -> Result<(), IamError> {
    if name.is_empty() {
        return Err(IamError::invalid_input("Entity name must not be empty"));
    }
    if name.len() > max_len {
        return Err(IamError::invalid_input(format!(
            "Entity name must not exceed {max_len} characters"
        )));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "+=,.@_-".contains(c))
    {
        return Err(IamError::invalid_input(format!(
            "Entity name '{name}' contains invalid characters. Only alphanumeric characters and \
             +=,.@_- are allowed."
        )));
    }
    Ok(())
}

/// Validate an IAM path.
///
/// Paths must start and end with `/`, and may only contain alphanumeric
/// characters plus `+=,.@_-/`.
///
/// # Errors
///
/// Returns [`IamError`] if the path does not start/end with `/` or
/// contains invalid characters.
pub fn validate_path(path: &str) -> Result<(), IamError> {
    if !path.starts_with('/') || !path.ends_with('/') {
        return Err(IamError::invalid_input("Path must begin and end with '/'"));
    }
    if !path
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "+=,.@_-/".contains(c))
    {
        return Err(IamError::invalid_input(format!(
            "Path '{path}' contains invalid characters"
        )));
    }
    Ok(())
}

/// Validate a policy document.
///
/// Checks that the document is valid JSON. Does not perform semantic
/// validation of the policy grammar.
///
/// # Errors
///
/// Returns [`IamError`] if the document is not valid JSON.
pub fn validate_policy_document(doc: &str) -> Result<(), IamError> {
    serde_json::from_str::<serde_json::Value>(doc)
        .map_err(|e| IamError::malformed_policy_document(format!("Invalid JSON: {e}")))?;
    Ok(())
}

/// Validate a role's maximum session duration.
///
/// Must be between 3600 (1 hour) and 43200 (12 hours) seconds.
///
/// # Errors
///
/// Returns [`IamError`] if the duration is out of range.
pub fn validate_max_session_duration(duration: i32) -> Result<(), IamError> {
    if !(3600..=43200).contains(&duration) {
        return Err(IamError::invalid_input(format!(
            "MaxSessionDuration must be between 3600 and 43200, got {duration}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_accept_valid_entity_name() {
        assert!(validate_entity_name("alice", 64).is_ok());
        assert!(validate_entity_name("my-role_v2", 64).is_ok());
        assert!(validate_entity_name("user@domain.com", 64).is_ok());
        assert!(validate_entity_name("a+=b,.c@d_e-f", 128).is_ok());
    }

    #[test]
    fn test_should_reject_empty_entity_name() {
        assert!(validate_entity_name("", 64).is_err());
    }

    #[test]
    fn test_should_reject_too_long_entity_name() {
        let long = "a".repeat(65);
        assert!(validate_entity_name(&long, 64).is_err());
    }

    #[test]
    fn test_should_reject_invalid_chars_in_entity_name() {
        assert!(validate_entity_name("user name", 64).is_err());
        assert!(validate_entity_name("user/name", 64).is_err());
    }

    #[test]
    fn test_should_accept_valid_path() {
        assert!(validate_path("/").is_ok());
        assert!(validate_path("/division/").is_ok());
        assert!(validate_path("/a/b/c/").is_ok());
    }

    #[test]
    fn test_should_reject_invalid_path() {
        assert!(validate_path("").is_err());
        assert!(validate_path("noslash").is_err());
        assert!(validate_path("/noslash").is_err());
    }

    #[test]
    fn test_should_accept_valid_policy_document() {
        assert!(validate_policy_document(r#"{"Version":"2012-10-17"}"#).is_ok());
    }

    #[test]
    fn test_should_reject_invalid_policy_document() {
        assert!(validate_policy_document("not json").is_err());
    }

    #[test]
    fn test_should_accept_valid_session_duration() {
        assert!(validate_max_session_duration(3600).is_ok());
        assert!(validate_max_session_duration(43200).is_ok());
        assert!(validate_max_session_duration(7200).is_ok());
    }

    #[test]
    fn test_should_reject_invalid_session_duration() {
        assert!(validate_max_session_duration(3599).is_err());
        assert!(validate_max_session_duration(43201).is_err());
    }
}
