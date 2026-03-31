//! Email store for retrospection via `/_aws/ses`.
//!
//! Captures all "sent" emails in an append-only store backed by `DashMap`.
//! No actual email delivery occurs -- all emails are captured in memory
//! for test inspection.

use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Store for all sent emails, enabling retrospection via `/_aws/ses`.
///
/// Append-only: emails are added when `SendEmail`, `SendRawEmail`, or
/// `SendTemplatedEmail` is called. Emails can be queried by message ID
/// or source address, and cleared for test isolation.
#[derive(Debug)]
pub struct EmailStore {
    /// All sent emails keyed by message ID.
    emails: DashMap<String, SentEmail>,
    /// Total number of emails sent (monotonically increasing, not reset on clear).
    total_sent: AtomicU64,
}

impl Default for EmailStore {
    fn default() -> Self {
        Self::new()
    }
}

impl EmailStore {
    /// Create a new empty email store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            emails: DashMap::new(),
            total_sent: AtomicU64::new(0),
        }
    }

    /// Store a sent email for retrospection. Returns the generated message ID.
    pub fn capture(&self, email: SentEmail) -> String {
        let id = email.id.clone();
        self.emails.insert(id.clone(), email);
        self.total_sent.fetch_add(1, Ordering::Relaxed);
        id
    }

    /// Query emails with optional filters.
    #[must_use]
    pub fn query(&self, filter_id: Option<&str>, filter_source: Option<&str>) -> Vec<SentEmail> {
        self.emails
            .iter()
            .filter(|entry| {
                let email = entry.value();
                let id_match = filter_id.is_none_or(|id| id.is_empty() || email.id == id);
                let source_match =
                    filter_source.is_none_or(|src| src.is_empty() || email.source == src);
                id_match && source_match
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Remove a specific email by message ID.
    pub fn remove(&self, id: &str) {
        self.emails.remove(id);
    }

    /// Clear all captured emails. Does NOT reset the `total_sent` counter.
    pub fn clear(&self) {
        self.emails.clear();
    }

    /// Get the total number of emails sent (lifetime, not reset on clear).
    #[must_use]
    pub fn total_sent(&self) -> u64 {
        self.total_sent.load(Ordering::Relaxed)
    }

    /// Get the current number of stored emails.
    #[must_use]
    pub fn count(&self) -> usize {
        self.emails.len()
    }
}

/// A single captured email, stored for retrospection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentEmail {
    /// Unique message ID (UUID).
    pub id: String,
    /// AWS region where the email was sent.
    pub region: String,
    /// ISO 8601 timestamp of when the email was captured.
    pub timestamp: String,
    /// Source (From) email address.
    pub source: String,
    /// Destination addresses.
    pub destination: SentEmailDestination,
    /// Email subject line (for `SendEmail`, `SendTemplatedEmail` after rendering).
    pub subject: Option<String>,
    /// Email body.
    pub body: Option<SentEmailBody>,
    /// Raw MIME data (for `SendRawEmail`).
    pub raw_data: Option<String>,
    /// Template name (for `SendTemplatedEmail`).
    pub template: Option<String>,
    /// Template data JSON string (for `SendTemplatedEmail`).
    pub template_data: Option<String>,
    /// Message tags from the send request.
    pub tags: Vec<SentEmailTag>,
}

/// Destination addresses for a sent email.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentEmailDestination {
    /// To addresses.
    pub to_addresses: Vec<String>,
    /// CC addresses.
    pub cc_addresses: Vec<String>,
    /// BCC addresses.
    pub bcc_addresses: Vec<String>,
}

/// Body of a sent email.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentEmailBody {
    /// Plain text body part.
    pub text_part: Option<String>,
    /// HTML body part.
    pub html_part: Option<String>,
}

/// A message tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentEmailTag {
    /// Tag name.
    pub name: String,
    /// Tag value.
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_email(id: &str, source: &str) -> SentEmail {
        SentEmail {
            id: id.to_owned(),
            region: "us-east-1".to_owned(),
            timestamp: "2026-03-19T10:00:00Z".to_owned(),
            source: source.to_owned(),
            destination: SentEmailDestination {
                to_addresses: vec!["recipient@example.com".to_owned()],
                cc_addresses: Vec::new(),
                bcc_addresses: Vec::new(),
            },
            subject: Some("Test Subject".to_owned()),
            body: Some(SentEmailBody {
                text_part: Some("Hello".to_owned()),
                html_part: None,
            }),
            raw_data: None,
            template: None,
            template_data: None,
            tags: Vec::new(),
        }
    }

    #[test]
    fn test_should_capture_and_query_email() {
        let store = EmailStore::new();
        let email = make_test_email("msg-1", "sender@example.com");
        let id = store.capture(email);
        assert_eq!(id, "msg-1");
        assert_eq!(store.count(), 1);
        assert_eq!(store.total_sent(), 1);

        let results = store.query(None, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, "sender@example.com");
    }

    #[test]
    fn test_should_query_by_id() {
        let store = EmailStore::new();
        store.capture(make_test_email("msg-1", "a@b.com"));
        store.capture(make_test_email("msg-2", "c@d.com"));

        let results = store.query(Some("msg-1"), None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "msg-1");
    }

    #[test]
    fn test_should_query_by_source() {
        let store = EmailStore::new();
        store.capture(make_test_email("msg-1", "a@b.com"));
        store.capture(make_test_email("msg-2", "c@d.com"));

        let results = store.query(None, Some("a@b.com"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, "a@b.com");
    }

    #[test]
    fn test_should_query_with_both_filters() {
        let store = EmailStore::new();
        store.capture(make_test_email("msg-1", "a@b.com"));
        store.capture(make_test_email("msg-2", "a@b.com"));

        let results = store.query(Some("msg-1"), Some("a@b.com"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "msg-1");
    }

    #[test]
    fn test_should_remove_single_email() {
        let store = EmailStore::new();
        store.capture(make_test_email("msg-1", "a@b.com"));
        store.capture(make_test_email("msg-2", "c@d.com"));
        store.remove("msg-1");
        assert_eq!(store.count(), 1);
        assert!(store.query(Some("msg-1"), None).is_empty());
    }

    #[test]
    fn test_should_clear_all_emails() {
        let store = EmailStore::new();
        store.capture(make_test_email("msg-1", "a@b.com"));
        store.capture(make_test_email("msg-2", "c@d.com"));
        store.clear();
        assert_eq!(store.count(), 0);
        // total_sent is NOT reset on clear
        assert_eq!(store.total_sent(), 2);
    }

    #[test]
    fn test_should_serialize_to_json() {
        let email = make_test_email("msg-1", "sender@example.com");
        let json = serde_json::to_string(&email);
        assert!(json.is_ok());
        let json = json.unwrap_or_default();
        assert!(json.contains("\"id\":\"msg-1\""));
        assert!(json.contains("\"source\":\"sender@example.com\""));
        assert!(json.contains("\"toAddresses\""));
    }
}
