//! S3 authentication via the `CredentialProvider` trait.
//!
//! [`RustackAuth`] implements the [`rustack_auth::credentials::CredentialProvider`]
//! trait to provide authentication for the Rustack S3 service. When signature
//! validation is skipped (the default for local development), any access key maps
//! to an empty secret key, effectively disabling signature verification.
//!
//! When validation is enabled, all access keys map to the secret key `"test"`,
//! matching LocalStack's default behavior.

use rustack_auth::{credentials::CredentialProvider, error::AuthError};
use tracing::debug;

/// Rustack authentication provider.
///
/// # Examples
///
/// ```
/// use rustack_s3_core::auth::RustackAuth;
///
/// let auth = RustackAuth::new(true);
/// assert!(auth.skip_validation());
/// ```
#[derive(Debug, Clone)]
pub struct RustackAuth {
    skip_validation: bool,
}

impl RustackAuth {
    /// Create a new authentication provider.
    ///
    /// When `skip_validation` is `true`, all signature checks are effectively
    /// bypassed by returning an empty secret key for any access key.
    #[must_use]
    pub fn new(skip_validation: bool) -> Self {
        Self { skip_validation }
    }

    /// Whether signature validation is skipped.
    #[must_use]
    pub fn skip_validation(&self) -> bool {
        self.skip_validation
    }
}

impl CredentialProvider for RustackAuth {
    fn get_secret_key(&self, access_key_id: &str) -> Result<String, AuthError> {
        if self.skip_validation {
            debug!(access_key_id, "Skipping signature validation");
            return Ok(String::new());
        }

        debug!(access_key_id, "Returning default secret key for access key");
        Ok("test".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_create_auth_with_skip_validation() {
        let auth = RustackAuth::new(true);
        assert!(auth.skip_validation());
    }

    #[test]
    fn test_should_create_auth_with_validation() {
        let auth = RustackAuth::new(false);
        assert!(!auth.skip_validation());
    }

    #[test]
    fn test_should_return_empty_key_when_skipping_validation() {
        let auth = RustackAuth::new(true);
        let key = auth.get_secret_key("any-key").expect("test get_secret_key");
        assert_eq!(key, "");
    }

    #[test]
    fn test_should_return_test_key_when_validating() {
        let auth = RustackAuth::new(false);
        let key = auth
            .get_secret_key("AKIAIOSFODNN7EXAMPLE")
            .expect("test get_secret_key");
        assert_eq!(key, "test");
    }
}
