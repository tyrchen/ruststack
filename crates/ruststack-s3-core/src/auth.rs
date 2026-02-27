//! S3 authentication via the `s3s::auth::S3Auth` trait.
//!
//! [`RustStackAuth`] implements the [`s3s::auth::S3Auth`] trait to provide
//! authentication for the RustStack S3 service. When signature validation is
//! skipped (the default for local development), any access key maps to an
//! empty secret key, effectively disabling signature verification.
//!
//! When validation is enabled, all access keys map to the secret key `"test"`,
//! matching LocalStack's default behavior.
//!
//! # Object safety
//!
//! The [`s3s::auth::S3Auth`] trait uses `#[async_trait]` because it must be
//! object-safe for dynamic dispatch (e.g. `Box<dyn S3Auth>`). We follow the
//! same pattern here.

use s3s::S3Result;
use s3s::auth::SecretKey;
use tracing::debug;

/// RustStack authentication provider.
///
/// # Examples
///
/// ```
/// use ruststack_s3_core::auth::RustStackAuth;
///
/// let auth = RustStackAuth::new(true);
/// assert!(auth.skip_validation());
/// ```
#[derive(Debug, Clone)]
pub struct RustStackAuth {
    skip_validation: bool,
}

impl RustStackAuth {
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

#[async_trait::async_trait]
impl s3s::auth::S3Auth for RustStackAuth {
    async fn get_secret_key(&self, access_key: &str) -> S3Result<SecretKey> {
        if self.skip_validation {
            debug!(access_key, "Skipping signature validation");
            return Ok(SecretKey::from(""));
        }

        debug!(access_key, "Returning default secret key for access key");
        Ok(SecretKey::from("test"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_create_auth_with_skip_validation() {
        let auth = RustStackAuth::new(true);
        assert!(auth.skip_validation());
    }

    #[test]
    fn test_should_create_auth_with_validation() {
        let auth = RustStackAuth::new(false);
        assert!(!auth.skip_validation());
    }

    #[tokio::test]
    async fn test_should_return_empty_key_when_skipping_validation() {
        use s3s::auth::S3Auth;

        let auth = RustStackAuth::new(true);
        let key = auth
            .get_secret_key("any-key")
            .await
            .expect("test get_secret_key");
        assert_eq!(key.expose(), "");
    }

    #[tokio::test]
    async fn test_should_return_test_key_when_validating() {
        use s3s::auth::S3Auth;

        let auth = RustStackAuth::new(false);
        let key = auth
            .get_secret_key("AKIAIOSFODNN7EXAMPLE")
            .await
            .expect("test get_secret_key");
        assert_eq!(key.expose(), "test");
    }
}
