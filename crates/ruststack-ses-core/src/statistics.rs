//! Send statistics and quota tracking.
//!
//! Tracks send counts, bounces, complaints, and delivery attempts using
//! atomic counters for lock-free concurrent access.

use std::sync::atomic::{AtomicU64, Ordering};

/// Tracks send statistics for `GetSendStatistics` and `GetSendQuota`.
#[derive(Debug)]
pub struct SendStatistics {
    /// Number of successful sends.
    send_count: AtomicU64,
    /// Number of bounces (always 0 in local dev).
    bounce_count: AtomicU64,
    /// Number of complaints (always 0 in local dev).
    complaint_count: AtomicU64,
    /// Number of delivery attempts.
    delivery_attempts: AtomicU64,
    /// Number of rejects (always 0 in local dev).
    reject_count: AtomicU64,
}

impl Default for SendStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl SendStatistics {
    /// Create a new `SendStatistics` with all counters at zero.
    #[must_use]
    pub fn new() -> Self {
        Self {
            send_count: AtomicU64::new(0),
            bounce_count: AtomicU64::new(0),
            complaint_count: AtomicU64::new(0),
            delivery_attempts: AtomicU64::new(0),
            reject_count: AtomicU64::new(0),
        }
    }

    /// Record a successful send.
    pub fn record_send(&self) {
        self.send_count.fetch_add(1, Ordering::Relaxed);
        self.delivery_attempts.fetch_add(1, Ordering::Relaxed);
    }

    /// Get a snapshot of current statistics.
    #[must_use]
    pub fn get_stats(&self) -> SendStats {
        SendStats {
            send_count: self.send_count.load(Ordering::Relaxed),
            bounce_count: self.bounce_count.load(Ordering::Relaxed),
            complaint_count: self.complaint_count.load(Ordering::Relaxed),
            delivery_attempts: self.delivery_attempts.load(Ordering::Relaxed),
            reject_count: self.reject_count.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of send statistics.
#[derive(Debug, Clone)]
pub struct SendStats {
    /// Number of successful sends.
    pub send_count: u64,
    /// Number of bounces.
    pub bounce_count: u64,
    /// Number of complaints.
    pub complaint_count: u64,
    /// Number of delivery attempts.
    pub delivery_attempts: u64,
    /// Number of rejects.
    pub reject_count: u64,
}

/// Send quota configuration.
#[derive(Debug, Clone)]
pub struct SendQuotaConfig {
    /// Max sends per 24 hours.
    pub max_24_hour_send: f64,
    /// Max sends per second.
    pub max_send_rate: f64,
}

impl Default for SendQuotaConfig {
    fn default() -> Self {
        Self {
            max_24_hour_send: 200.0,
            max_send_rate: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_start_with_zero_counts() {
        let stats = SendStatistics::new();
        let snapshot = stats.get_stats();
        assert_eq!(snapshot.send_count, 0);
        assert_eq!(snapshot.bounce_count, 0);
        assert_eq!(snapshot.complaint_count, 0);
        assert_eq!(snapshot.delivery_attempts, 0);
        assert_eq!(snapshot.reject_count, 0);
    }

    #[test]
    fn test_should_increment_on_send() {
        let stats = SendStatistics::new();
        stats.record_send();
        stats.record_send();
        let snapshot = stats.get_stats();
        assert_eq!(snapshot.send_count, 2);
        assert_eq!(snapshot.delivery_attempts, 2);
        assert_eq!(snapshot.bounce_count, 0);
    }

    #[test]
    fn test_should_create_default_quota() {
        let quota = SendQuotaConfig::default();
        assert!((quota.max_24_hour_send - 200.0).abs() < f64::EPSILON);
        assert!((quota.max_send_rate - 1.0).abs() < f64::EPSILON);
    }
}
