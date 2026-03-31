//! IAM ARN generation.
//!
//! Builds Amazon Resource Names for IAM entities following the
//! `arn:aws:iam::{account_id}:{resource_type}/{path}{name}` format.

/// Build an IAM ARN.
///
/// - Default path `"/"` produces `arn:aws:iam::{account_id}:{resource_type}/{name}`
/// - Custom path `"/dev/"` produces `arn:aws:iam::{account_id}:{resource_type}/dev/{name}`
///
/// # Examples
///
/// ```
/// use rustack_iam_core::arn::iam_arn;
///
/// let arn = iam_arn("123456789012", "user", "/", "alice");
/// assert_eq!(arn, "arn:aws:iam::123456789012:user/alice");
///
/// let arn = iam_arn("123456789012", "user", "/dev/", "bob");
/// assert_eq!(arn, "arn:aws:iam::123456789012:user/dev/bob");
/// ```
#[must_use]
pub fn iam_arn(account_id: &str, resource_type: &str, path: &str, name: &str) -> String {
    if path == "/" {
        format!("arn:aws:iam::{account_id}:{resource_type}/{name}")
    } else {
        // Strip leading slash for ARN construction.
        let trimmed = path.strip_prefix('/').unwrap_or(path);
        format!("arn:aws:iam::{account_id}:{resource_type}/{trimmed}{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_generate_arn_with_default_path() {
        let arn = iam_arn("123456789012", "user", "/", "alice");
        assert_eq!(arn, "arn:aws:iam::123456789012:user/alice");
    }

    #[test]
    fn test_should_generate_arn_with_custom_path() {
        let arn = iam_arn("123456789012", "role", "/application/", "my-role");
        assert_eq!(arn, "arn:aws:iam::123456789012:role/application/my-role");
    }

    #[test]
    fn test_should_generate_policy_arn() {
        let arn = iam_arn("123456789012", "policy", "/", "MyPolicy");
        assert_eq!(arn, "arn:aws:iam::123456789012:policy/MyPolicy");
    }

    #[test]
    fn test_should_generate_instance_profile_arn() {
        let arn = iam_arn("123456789012", "instance-profile", "/", "MyProfile");
        assert_eq!(arn, "arn:aws:iam::123456789012:instance-profile/MyProfile");
    }
}
