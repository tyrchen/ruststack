//! Queue actor and storage.
//!
//! Each SQS queue runs as an independent actor that owns its message state
//! and communicates via `tokio::sync::mpsc` channels. This follows the actor
//! model mandated by the project conventions.

pub(crate) mod actor;
pub(crate) mod attributes;
pub(crate) mod storage;
pub(crate) mod url;

pub use actor::{QueueActor, QueueCommand, QueueHandle, QueueMetadata};
pub use attributes::QueueAttributes;
pub use storage::StandardQueueStorage;
pub use url::{extract_queue_name, queue_arn, queue_url};
