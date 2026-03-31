//! SSM output types for Phase 0, Phase 1, and Phase 2 operations.
//!
//! All output structs use `PascalCase` JSON field naming to match the SSM
//! wire protocol (`awsJson1_1`). Optional fields are omitted when `None`.

use serde::{Deserialize, Serialize};

use crate::types::{Parameter, ParameterHistory, ParameterMetadata, Tag};

// ---------------------------------------------------------------------------
// PutParameter
// ---------------------------------------------------------------------------

/// Output for the `PutParameter` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PutParameterOutput {
    /// The version of the parameter.
    pub version: i64,

    /// The tier of the parameter.
    pub tier: String,
}

// ---------------------------------------------------------------------------
// GetParameter
// ---------------------------------------------------------------------------

/// Output for the `GetParameter` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetParameterOutput {
    /// The parameter details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter: Option<Parameter>,
}

// ---------------------------------------------------------------------------
// GetParameters
// ---------------------------------------------------------------------------

/// Output for the `GetParameters` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetParametersOutput {
    /// The parameters that were found.
    #[serde(default)]
    pub parameters: Vec<Parameter>,

    /// The names of parameters that could not be found.
    #[serde(default)]
    pub invalid_parameters: Vec<String>,
}

// ---------------------------------------------------------------------------
// GetParametersByPath
// ---------------------------------------------------------------------------

/// Output for the `GetParametersByPath` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetParametersByPathOutput {
    /// The parameters that match the path.
    #[serde(default)]
    pub parameters: Vec<Parameter>,

    /// The token for the next page of results, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

// ---------------------------------------------------------------------------
// DeleteParameter
// ---------------------------------------------------------------------------

/// Output for the `DeleteParameter` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteParameterOutput {}

// ---------------------------------------------------------------------------
// DeleteParameters
// ---------------------------------------------------------------------------

/// Output for the `DeleteParameters` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteParametersOutput {
    /// The names of parameters that were successfully deleted.
    #[serde(default)]
    pub deleted_parameters: Vec<String>,

    /// The names of parameters that could not be found.
    #[serde(default)]
    pub invalid_parameters: Vec<String>,
}

// ---------------------------------------------------------------------------
// Phase 1: DescribeParameters
// ---------------------------------------------------------------------------

/// Output for the `DescribeParameters` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribeParametersOutput {
    /// The parameter metadata entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ParameterMetadata>,

    /// The token for the next page of results, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Phase 1: GetParameterHistory
// ---------------------------------------------------------------------------

/// Output for the `GetParameterHistory` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetParameterHistoryOutput {
    /// The version history entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ParameterHistory>,

    /// The token for the next page of results, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Phase 1: AddTagsToResource
// ---------------------------------------------------------------------------

/// Output for the `AddTagsToResource` operation (empty).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AddTagsToResourceOutput {}

// ---------------------------------------------------------------------------
// Phase 1: RemoveTagsFromResource
// ---------------------------------------------------------------------------

/// Output for the `RemoveTagsFromResource` operation (empty).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RemoveTagsFromResourceOutput {}

// ---------------------------------------------------------------------------
// Phase 1: ListTagsForResource
// ---------------------------------------------------------------------------

/// Output for the `ListTagsForResource` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListTagsForResourceOutput {
    /// The tags associated with the resource.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tag_list: Vec<Tag>,
}

// ---------------------------------------------------------------------------
// Phase 2: LabelParameterVersion
// ---------------------------------------------------------------------------

/// Output for the `LabelParameterVersion` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LabelParameterVersionOutput {
    /// Labels that failed validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invalid_labels: Vec<String>,

    /// The version number that was labeled.
    pub parameter_version: i64,
}

// ---------------------------------------------------------------------------
// Phase 2: UnlabelParameterVersion
// ---------------------------------------------------------------------------

/// Output for the `UnlabelParameterVersion` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UnlabelParameterVersionOutput {
    /// Labels that were not found on the specified version.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invalid_labels: Vec<String>,

    /// Labels that were successfully removed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_labels: Vec<String>,
}
