//! Permission operation output types.

use serde::{Deserialize, Serialize};

/// Output for `AddPermission` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AddPermissionOutput {}

/// Output for `RemovePermission` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RemovePermissionOutput {}
