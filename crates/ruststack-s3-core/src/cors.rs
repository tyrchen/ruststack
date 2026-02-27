//! CORS rule matching and response header generation.
//!
//! Provides [`CorsIndex`] for storing per-bucket CORS configurations and
//! matching incoming requests against those rules. The matching logic follows
//! the S3 CORS specification, including wildcard origin support and preflight
//! request handling.

use dashmap::DashMap;

// ---------------------------------------------------------------------------
// CorsRule
// ---------------------------------------------------------------------------

/// A single CORS configuration rule for an S3 bucket.
///
/// Each rule specifies which origins, methods, and headers are allowed, which
/// response headers may be exposed, and how long the browser may cache the
/// preflight result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorsRule {
    /// Origins that are allowed (supports `"*"` wildcard).
    pub allowed_origins: Vec<String>,
    /// HTTP methods that are allowed (e.g. `"GET"`, `"PUT"`).
    pub allowed_methods: Vec<String>,
    /// Request headers that are allowed (supports `"*"` wildcard).
    pub allowed_headers: Vec<String>,
    /// Response headers that the browser is allowed to access.
    pub expose_headers: Vec<String>,
    /// How long (in seconds) the browser may cache the preflight result.
    pub max_age_seconds: Option<i32>,
}

// ---------------------------------------------------------------------------
// CorsMatch
// ---------------------------------------------------------------------------

/// The result of a successful CORS rule match.
///
/// Contains the values that should be included in the CORS response headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorsMatch {
    /// The allowed origin for `Access-Control-Allow-Origin`.
    pub allowed_origin: String,
    /// The allowed methods for `Access-Control-Allow-Methods`.
    pub allowed_methods: Vec<String>,
    /// The allowed headers for `Access-Control-Allow-Headers`.
    pub allowed_headers: Vec<String>,
    /// The headers to expose via `Access-Control-Expose-Headers`.
    pub expose_headers: Vec<String>,
    /// Max age for `Access-Control-Max-Age`.
    pub max_age_seconds: Option<i32>,
}

// ---------------------------------------------------------------------------
// CorsIndex
// ---------------------------------------------------------------------------

/// Thread-safe, per-bucket CORS rule index.
///
/// Uses [`DashMap`] for lock-free concurrent reads and writes.
///
/// # Examples
///
/// ```
/// use ruststack_s3_core::cors::{CorsIndex, CorsRule};
///
/// let index = CorsIndex::new();
/// index.set_rules("my-bucket", vec![
///     CorsRule {
///         allowed_origins: vec!["*".to_owned()],
///         allowed_methods: vec!["GET".to_owned()],
///         allowed_headers: vec![],
///         expose_headers: vec![],
///         max_age_seconds: None,
///     },
/// ]);
///
/// let m = index.match_cors("my-bucket", "https://example.com", "GET");
/// assert!(m.is_some());
/// ```
#[derive(Debug)]
pub struct CorsIndex {
    rules: DashMap<String, Vec<CorsRule>>,
}

impl CorsIndex {
    /// Create a new empty CORS index.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: DashMap::new(),
        }
    }

    /// Set CORS rules for a bucket, replacing any existing rules.
    pub fn set_rules(&self, bucket: &str, rules: Vec<CorsRule>) {
        self.rules.insert(bucket.to_owned(), rules);
    }

    /// Delete all CORS rules for a bucket.
    pub fn delete_rules(&self, bucket: &str) {
        self.rules.remove(bucket);
    }

    /// Get a clone of the CORS rules for a bucket.
    #[must_use]
    pub fn get_rules(&self, bucket: &str) -> Option<Vec<CorsRule>> {
        self.rules.get(bucket).map(|r| r.value().clone())
    }

    /// Match an actual (non-preflight) request against the bucket's CORS rules.
    ///
    /// Returns the first matching [`CorsMatch`] or `None` if no rule matches.
    #[must_use]
    pub fn match_cors(&self, bucket: &str, origin: &str, method: &str) -> Option<CorsMatch> {
        let rules = self.rules.get(bucket)?;
        for rule in rules.value() {
            if !rule.allowed_origins.iter().any(|p| match_origin(p, origin)) {
                continue;
            }
            if !rule
                .allowed_methods
                .iter()
                .any(|m| m.eq_ignore_ascii_case(method))
            {
                continue;
            }
            return Some(CorsMatch {
                allowed_origin: resolve_origin(&rule.allowed_origins, origin),
                allowed_methods: rule.allowed_methods.clone(),
                allowed_headers: rule.allowed_headers.clone(),
                expose_headers: rule.expose_headers.clone(),
                max_age_seconds: rule.max_age_seconds,
            });
        }
        None
    }

    /// Match a preflight (OPTIONS) request against the bucket's CORS rules.
    ///
    /// Returns the first matching [`CorsMatch`] or `None` if no rule matches.
    /// Unlike [`match_cors`](Self::match_cors), this also validates the
    /// `Access-Control-Request-Headers` against `allowed_headers`.
    #[must_use]
    pub fn match_preflight(
        &self,
        bucket: &str,
        origin: &str,
        request_method: &str,
        request_headers: &[String],
    ) -> Option<CorsMatch> {
        let rules = self.rules.get(bucket)?;
        for rule in rules.value() {
            if !rule.allowed_origins.iter().any(|p| match_origin(p, origin)) {
                continue;
            }
            if !rule
                .allowed_methods
                .iter()
                .any(|m| m.eq_ignore_ascii_case(request_method))
            {
                continue;
            }
            if !headers_allowed(&rule.allowed_headers, request_headers) {
                continue;
            }
            return Some(CorsMatch {
                allowed_origin: resolve_origin(&rule.allowed_origins, origin),
                allowed_methods: rule.allowed_methods.clone(),
                allowed_headers: rule.allowed_headers.clone(),
                expose_headers: rule.expose_headers.clone(),
                max_age_seconds: rule.max_age_seconds,
            });
        }
        None
    }
}

impl Default for CorsIndex {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Matching helpers
// ---------------------------------------------------------------------------

/// Match an origin pattern against an actual origin.
///
/// A pattern of `"*"` matches any origin. Otherwise the comparison is
/// case-sensitive and exact.
#[must_use]
pub fn match_origin(pattern: &str, origin: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    pattern == origin
}

/// Determine the effective `Access-Control-Allow-Origin` value.
///
/// If any allowed origin is `"*"`, the header is `"*"`. Otherwise the
/// requesting origin is echoed back.
fn resolve_origin(allowed_origins: &[String], origin: &str) -> String {
    if allowed_origins.iter().any(|o| o == "*") {
        "*".to_owned()
    } else {
        origin.to_owned()
    }
}

/// Check whether all requested headers are permitted by the rule's
/// `allowed_headers`.
fn headers_allowed(allowed: &[String], requested: &[String]) -> bool {
    if allowed.iter().any(|h| h == "*") {
        return true;
    }
    requested
        .iter()
        .all(|req| allowed.iter().any(|a| a.eq_ignore_ascii_case(req)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_permissive_rule() -> CorsRule {
        CorsRule {
            allowed_origins: vec!["*".to_owned()],
            allowed_methods: vec!["GET".to_owned(), "PUT".to_owned(), "POST".to_owned()],
            allowed_headers: vec!["*".to_owned()],
            expose_headers: vec!["x-amz-request-id".to_owned()],
            max_age_seconds: Some(3600),
        }
    }

    fn make_strict_rule() -> CorsRule {
        CorsRule {
            allowed_origins: vec!["https://example.com".to_owned()],
            allowed_methods: vec!["GET".to_owned()],
            allowed_headers: vec!["Content-Type".to_owned()],
            expose_headers: vec![],
            max_age_seconds: None,
        }
    }

    // -----------------------------------------------------------------------
    // CorsIndex basic operations
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_set_and_get_rules() {
        let index = CorsIndex::new();
        let rules = vec![make_permissive_rule()];
        index.set_rules("bucket-a", rules.clone());

        let got = index.get_rules("bucket-a");
        assert!(got.is_some());
        assert_eq!(got.expect("test get"), rules);
    }

    #[test]
    fn test_should_return_none_for_unknown_bucket() {
        let index = CorsIndex::new();
        assert!(index.get_rules("nonexistent").is_none());
    }

    #[test]
    fn test_should_delete_rules() {
        let index = CorsIndex::new();
        index.set_rules("bucket-a", vec![make_permissive_rule()]);
        index.delete_rules("bucket-a");
        assert!(index.get_rules("bucket-a").is_none());
    }

    #[test]
    fn test_should_replace_existing_rules() {
        let index = CorsIndex::new();
        index.set_rules("bucket-a", vec![make_permissive_rule()]);
        index.set_rules("bucket-a", vec![make_strict_rule()]);

        let got = index.get_rules("bucket-a").expect("test get");
        assert_eq!(got.len(), 1);
        assert_eq!(
            got[0].allowed_origins,
            vec!["https://example.com".to_owned()],
        );
    }

    // -----------------------------------------------------------------------
    // match_cors
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_match_wildcard_origin() {
        let index = CorsIndex::new();
        index.set_rules("bucket", vec![make_permissive_rule()]);

        let m = index
            .match_cors("bucket", "https://any.example.com", "GET")
            .expect("test match");
        assert_eq!(m.allowed_origin, "*");
    }

    #[test]
    fn test_should_match_specific_origin() {
        let index = CorsIndex::new();
        index.set_rules("bucket", vec![make_strict_rule()]);

        let m = index
            .match_cors("bucket", "https://example.com", "GET")
            .expect("test match");
        assert_eq!(m.allowed_origin, "https://example.com");
    }

    #[test]
    fn test_should_not_match_wrong_origin() {
        let index = CorsIndex::new();
        index.set_rules("bucket", vec![make_strict_rule()]);

        assert!(
            index
                .match_cors("bucket", "https://evil.com", "GET")
                .is_none()
        );
    }

    #[test]
    fn test_should_not_match_wrong_method() {
        let index = CorsIndex::new();
        index.set_rules("bucket", vec![make_strict_rule()]);

        assert!(
            index
                .match_cors("bucket", "https://example.com", "DELETE")
                .is_none()
        );
    }

    #[test]
    fn test_should_not_match_unknown_bucket() {
        let index = CorsIndex::new();
        assert!(
            index
                .match_cors("nope", "https://example.com", "GET")
                .is_none()
        );
    }

    // -----------------------------------------------------------------------
    // match_preflight
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_match_preflight_with_wildcard_headers() {
        let index = CorsIndex::new();
        index.set_rules("bucket", vec![make_permissive_rule()]);

        let m = index
            .match_preflight(
                "bucket",
                "https://example.com",
                "PUT",
                &["X-Custom-Header".to_owned()],
            )
            .expect("test match");
        assert_eq!(m.allowed_origin, "*");
        assert!(m.max_age_seconds.is_some());
    }

    #[test]
    fn test_should_match_preflight_with_specific_headers() {
        let index = CorsIndex::new();
        index.set_rules("bucket", vec![make_strict_rule()]);

        let m = index
            .match_preflight(
                "bucket",
                "https://example.com",
                "GET",
                &["Content-Type".to_owned()],
            )
            .expect("test match");
        assert_eq!(m.allowed_origin, "https://example.com");
    }

    #[test]
    fn test_should_not_match_preflight_with_disallowed_header() {
        let index = CorsIndex::new();
        index.set_rules("bucket", vec![make_strict_rule()]);

        assert!(
            index
                .match_preflight(
                    "bucket",
                    "https://example.com",
                    "GET",
                    &["X-Forbidden".to_owned()],
                )
                .is_none()
        );
    }

    // -----------------------------------------------------------------------
    // match_origin helper
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_match_wildcard_pattern() {
        assert!(match_origin("*", "https://anything.com"));
    }

    #[test]
    fn test_should_match_exact_pattern() {
        assert!(match_origin("https://example.com", "https://example.com"));
    }

    #[test]
    fn test_should_not_match_different_pattern() {
        assert!(!match_origin("https://example.com", "https://other.com"));
    }
}
