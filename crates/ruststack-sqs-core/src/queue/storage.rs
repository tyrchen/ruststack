//! Standard queue message storage.

use std::collections::{HashMap, VecDeque};

use tokio::time::Instant;

use crate::message::{InFlightMessage, QueueMessage};

/// Standard queue storage with best-effort ordering and at-least-once delivery.
#[derive(Debug, Default)]
pub struct StandardQueueStorage {
    /// Available messages in approximate FIFO order.
    pub available: VecDeque<QueueMessage>,
    /// Messages not yet available (per-message or queue-level delay).
    pub delayed: Vec<QueueMessage>,
    /// Messages currently being processed by consumers.
    pub in_flight: HashMap<String, InFlightMessage>,
}

impl StandardQueueStorage {
    /// Promote delayed messages that have become available.
    pub fn promote_delayed(&mut self) -> bool {
        let now = Instant::now();
        let mut promoted = false;

        self.delayed.retain(|msg| {
            if msg.available_at <= now {
                promoted = true;
                false // Remove from delayed
            } else {
                true // Keep in delayed
            }
        });

        // Re-scan and move promoted messages. We need a second pass
        // to actually move them, since retain doesn't give ownership.
        let mut still_delayed = Vec::new();
        let old_delayed = std::mem::take(&mut self.delayed);
        for msg in old_delayed {
            if msg.available_at <= now {
                self.available.push_back(msg);
                promoted = true;
            } else {
                still_delayed.push(msg);
            }
        }
        self.delayed = still_delayed;

        promoted
    }

    /// Return expired in-flight messages to the available queue.
    pub fn return_expired_inflight(&mut self) -> bool {
        let now = Instant::now();
        let expired: Vec<String> = self
            .in_flight
            .iter()
            .filter(|(_, ifm)| ifm.visible_at <= now)
            .map(|(handle, _)| handle.clone())
            .collect();

        let had_expired = !expired.is_empty();
        for handle in expired {
            if let Some(ifm) = self.in_flight.remove(&handle) {
                self.available.push_back(ifm.message);
            }
        }
        had_expired
    }

    /// Get approximate message counts: (available, in_flight, delayed).
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // SQS queues are bounded well below u32::MAX.
    pub fn counts(&self) -> (u32, u32, u32) {
        (
            self.available.len() as u32,
            self.in_flight.len() as u32,
            self.delayed.len() as u32,
        )
    }

    /// Purge all messages.
    pub fn purge(&mut self) {
        self.available.clear();
        self.delayed.clear();
        self.in_flight.clear();
    }
}
