//! Permission operation input types.

use serde::{Deserialize, Serialize};

/// Input for `AddPermission`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AddPermissionInput {
    /// The queue URL.
    pub queue_url: String,
    /// A unique label for this permission statement.
    pub label: String,
    /// AWS account IDs to grant permission to.
    #[serde(rename = "AWSAccountIds")]
    pub aws_account_ids: Vec<String>,
    /// SQS actions to allow.
    pub actions: Vec<String>,
}

/// Input for `RemovePermission`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RemovePermissionInput {
    /// The queue URL.
    pub queue_url: String,
    /// The label of the permission statement to remove.
    pub label: String,
}
