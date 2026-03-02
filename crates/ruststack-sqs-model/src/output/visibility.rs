//! Visibility timeout output types.

use serde::{Deserialize, Serialize};

/// Output for `ChangeMessageVisibility` (empty response).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChangeMessageVisibilityOutput {}
