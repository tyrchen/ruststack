//! Publisher traits for cross-service message delivery.
//!
//! `SqsPublisher` is defined here (in sns-core) and implemented
//! in the server binary wrapping the actual SQS provider.
//! This keeps sns-core decoupled from sqs-core.
//!
//! Uses `async-trait` because `SqsPublisher` requires object safety
//! (used as `Arc<dyn SqsPublisher>` for dynamic dispatch).

use async_trait::async_trait;

/// Error type for message delivery failures.
#[derive(Debug, thiserror::Error)]
pub enum DeliveryError {
    /// SQS delivery failed.
    #[error("SQS delivery failed to {queue_arn}: {reason}")]
    SqsDeliveryFailed {
        /// The ARN of the target queue.
        queue_arn: String,
        /// The underlying error message.
        reason: String,
    },
    /// HTTP delivery failed.
    #[error("HTTP delivery failed to {endpoint}: {reason}")]
    HttpDeliveryFailed {
        /// The HTTP endpoint.
        endpoint: String,
        /// The underlying error message.
        reason: String,
    },
}

/// Trait for delivering messages to SQS queues.
///
/// Implemented by the server binary to bridge SNS fan-out to actual SQS queues.
#[async_trait]
pub trait SqsPublisher: Send + Sync + 'static {
    /// Send a message to an SQS queue identified by its ARN.
    async fn send_message(
        &self,
        queue_arn: &str,
        message_body: &str,
        message_group_id: Option<&str>,
        message_deduplication_id: Option<&str>,
    ) -> Result<(), DeliveryError>;
}

/// No-op publisher for testing or when SQS is not available.
#[derive(Debug)]
pub struct NoopSqsPublisher;

#[async_trait]
impl SqsPublisher for NoopSqsPublisher {
    async fn send_message(
        &self,
        _queue_arn: &str,
        _message_body: &str,
        _message_group_id: Option<&str>,
        _message_deduplication_id: Option<&str>,
    ) -> Result<(), DeliveryError> {
        Ok(())
    }
}
