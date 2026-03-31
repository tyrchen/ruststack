//! SQS operation enum.

use std::fmt;

/// All supported SQS operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SqsOperation {
    // Queue management
    /// Create a new queue.
    CreateQueue,
    /// Delete a queue.
    DeleteQueue,
    /// Get queue URL by name.
    GetQueueUrl,
    /// List queues.
    ListQueues,
    /// Get queue attributes.
    GetQueueAttributes,
    /// Set queue attributes.
    SetQueueAttributes,
    /// Purge all messages from a queue.
    PurgeQueue,

    // Message operations
    /// Send a message.
    SendMessage,
    /// Send a batch of messages.
    SendMessageBatch,
    /// Receive messages.
    ReceiveMessage,
    /// Delete a message.
    DeleteMessage,
    /// Delete a batch of messages.
    DeleteMessageBatch,
    /// Change message visibility timeout.
    ChangeMessageVisibility,
    /// Change visibility timeout for a batch of messages.
    ChangeMessageVisibilityBatch,

    // Tags
    /// Add tags to a queue.
    TagQueue,
    /// Remove tags from a queue.
    UntagQueue,
    /// List tags for a queue.
    ListQueueTags,

    // Permissions
    /// Add a permission to a queue.
    AddPermission,
    /// Remove a permission from a queue.
    RemovePermission,

    // Dead-letter queue
    /// List queues that have a dead-letter queue targeting a given queue.
    ListDeadLetterSourceQueues,
    /// Start a message move task from DLQ to source queue.
    StartMessageMoveTask,
    /// Cancel a message move task.
    CancelMessageMoveTask,
    /// List message move tasks.
    ListMessageMoveTasks,
}

impl SqsOperation {
    /// Returns the AWS operation name string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CreateQueue => "CreateQueue",
            Self::DeleteQueue => "DeleteQueue",
            Self::GetQueueUrl => "GetQueueUrl",
            Self::ListQueues => "ListQueues",
            Self::GetQueueAttributes => "GetQueueAttributes",
            Self::SetQueueAttributes => "SetQueueAttributes",
            Self::PurgeQueue => "PurgeQueue",
            Self::SendMessage => "SendMessage",
            Self::SendMessageBatch => "SendMessageBatch",
            Self::ReceiveMessage => "ReceiveMessage",
            Self::DeleteMessage => "DeleteMessage",
            Self::DeleteMessageBatch => "DeleteMessageBatch",
            Self::ChangeMessageVisibility => "ChangeMessageVisibility",
            Self::ChangeMessageVisibilityBatch => "ChangeMessageVisibilityBatch",
            Self::TagQueue => "TagQueue",
            Self::UntagQueue => "UntagQueue",
            Self::ListQueueTags => "ListQueueTags",
            Self::AddPermission => "AddPermission",
            Self::RemovePermission => "RemovePermission",
            Self::ListDeadLetterSourceQueues => "ListDeadLetterSourceQueues",
            Self::StartMessageMoveTask => "StartMessageMoveTask",
            Self::CancelMessageMoveTask => "CancelMessageMoveTask",
            Self::ListMessageMoveTasks => "ListMessageMoveTasks",
        }
    }

    /// Parse an operation name string into an [`SqsOperation`].
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "CreateQueue" => Some(Self::CreateQueue),
            "DeleteQueue" => Some(Self::DeleteQueue),
            "GetQueueUrl" => Some(Self::GetQueueUrl),
            "ListQueues" => Some(Self::ListQueues),
            "GetQueueAttributes" => Some(Self::GetQueueAttributes),
            "SetQueueAttributes" => Some(Self::SetQueueAttributes),
            "PurgeQueue" => Some(Self::PurgeQueue),
            "SendMessage" => Some(Self::SendMessage),
            "SendMessageBatch" => Some(Self::SendMessageBatch),
            "ReceiveMessage" => Some(Self::ReceiveMessage),
            "DeleteMessage" => Some(Self::DeleteMessage),
            "DeleteMessageBatch" => Some(Self::DeleteMessageBatch),
            "ChangeMessageVisibility" => Some(Self::ChangeMessageVisibility),
            "ChangeMessageVisibilityBatch" => Some(Self::ChangeMessageVisibilityBatch),
            "TagQueue" => Some(Self::TagQueue),
            "UntagQueue" => Some(Self::UntagQueue),
            "ListQueueTags" => Some(Self::ListQueueTags),
            "AddPermission" => Some(Self::AddPermission),
            "RemovePermission" => Some(Self::RemovePermission),
            "ListDeadLetterSourceQueues" => Some(Self::ListDeadLetterSourceQueues),
            "StartMessageMoveTask" => Some(Self::StartMessageMoveTask),
            "CancelMessageMoveTask" => Some(Self::CancelMessageMoveTask),
            "ListMessageMoveTasks" => Some(Self::ListMessageMoveTasks),
            _ => None,
        }
    }
}

impl fmt::Display for SqsOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
