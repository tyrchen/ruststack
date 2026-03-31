//! Auto-generated from AWS IAM Smithy model. DO NOT EDIT.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// IAM ContextKeyTypeEnum enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ContextKeyTypeEnum {
    /// Default variant.
    #[default]
    #[serde(rename = "binary")]
    Binary,
    #[serde(rename = "binaryList")]
    BinaryList,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "booleanList")]
    BooleanList,
    #[serde(rename = "date")]
    Date,
    #[serde(rename = "dateList")]
    DateList,
    #[serde(rename = "ip")]
    Ip,
    #[serde(rename = "ipList")]
    IpList,
    #[serde(rename = "numeric")]
    Numeric,
    #[serde(rename = "numericList")]
    NumericList,
    #[serde(rename = "string")]
    String,
    #[serde(rename = "stringList")]
    StringList,
}

impl ContextKeyTypeEnum {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Binary => "binary",
            Self::BinaryList => "binaryList",
            Self::Boolean => "boolean",
            Self::BooleanList => "booleanList",
            Self::Date => "date",
            Self::DateList => "dateList",
            Self::Ip => "ip",
            Self::IpList => "ipList",
            Self::Numeric => "numeric",
            Self::NumericList => "numericList",
            Self::String => "string",
            Self::StringList => "stringList",
        }
    }
}

impl std::fmt::Display for ContextKeyTypeEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for ContextKeyTypeEnum {
    fn from(s: &str) -> Self {
        match s {
            "binary" => Self::Binary,
            "binaryList" => Self::BinaryList,
            "boolean" => Self::Boolean,
            "booleanList" => Self::BooleanList,
            "date" => Self::Date,
            "dateList" => Self::DateList,
            "ip" => Self::Ip,
            "ipList" => Self::IpList,
            "numeric" => Self::Numeric,
            "numericList" => Self::NumericList,
            "string" => Self::String,
            "stringList" => Self::StringList,
            _ => Self::default(),
        }
    }
}

/// IAM DeletionTaskStatusType enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DeletionTaskStatusType {
    /// Default variant.
    #[default]
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "IN_PROGRESS")]
    InProgress,
    #[serde(rename = "NOT_STARTED")]
    NotStarted,
    #[serde(rename = "SUCCEEDED")]
    Succeeded,
}

impl DeletionTaskStatusType {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Failed => "FAILED",
            Self::InProgress => "IN_PROGRESS",
            Self::NotStarted => "NOT_STARTED",
            Self::Succeeded => "SUCCEEDED",
        }
    }
}

impl std::fmt::Display for DeletionTaskStatusType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for DeletionTaskStatusType {
    fn from(s: &str) -> Self {
        match s {
            "FAILED" => Self::Failed,
            "IN_PROGRESS" => Self::InProgress,
            "NOT_STARTED" => Self::NotStarted,
            "SUCCEEDED" => Self::Succeeded,
            _ => Self::default(),
        }
    }
}

/// IAM EntityType enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum EntityType {
    /// Default variant.
    #[default]
    #[serde(rename = "AWSManagedPolicy")]
    AwsManagedPolicy,
    Group,
    LocalManagedPolicy,
    Role,
    User,
}

impl EntityType {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AwsManagedPolicy => "AWSManagedPolicy",
            Self::Group => "Group",
            Self::LocalManagedPolicy => "LocalManagedPolicy",
            Self::Role => "Role",
            Self::User => "User",
        }
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for EntityType {
    fn from(s: &str) -> Self {
        match s {
            "AWSManagedPolicy" => Self::AwsManagedPolicy,
            "Group" => Self::Group,
            "LocalManagedPolicy" => Self::LocalManagedPolicy,
            "Role" => Self::Role,
            "User" => Self::User,
            _ => Self::default(),
        }
    }
}

/// IAM PermissionsBoundaryAttachmentType enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PermissionsBoundaryAttachmentType {
    /// Default variant.
    #[default]
    #[serde(rename = "PermissionsBoundaryPolicy")]
    Policy,
}

impl PermissionsBoundaryAttachmentType {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Policy => "PermissionsBoundaryPolicy",
        }
    }
}

impl std::fmt::Display for PermissionsBoundaryAttachmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for PermissionsBoundaryAttachmentType {
    fn from(s: &str) -> Self {
        match s {
            "PermissionsBoundaryPolicy" => Self::Policy,
            _ => Self::default(),
        }
    }
}

/// IAM PolicyEvaluationDecisionType enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PolicyEvaluationDecisionType {
    /// Default variant.
    #[default]
    #[serde(rename = "allowed")]
    Allowed,
    #[serde(rename = "explicitDeny")]
    ExplicitDeny,
    #[serde(rename = "implicitDeny")]
    ImplicitDeny,
}

impl PolicyEvaluationDecisionType {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::ExplicitDeny => "explicitDeny",
            Self::ImplicitDeny => "implicitDeny",
        }
    }
}

impl std::fmt::Display for PolicyEvaluationDecisionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for PolicyEvaluationDecisionType {
    fn from(s: &str) -> Self {
        match s {
            "allowed" => Self::Allowed,
            "explicitDeny" => Self::ExplicitDeny,
            "implicitDeny" => Self::ImplicitDeny,
            _ => Self::default(),
        }
    }
}

/// IAM PolicySourceType enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PolicySourceType {
    /// Default variant.
    #[default]
    #[serde(rename = "aws-managed")]
    AwsManaged,
    #[serde(rename = "group")]
    Group,
    #[serde(rename = "none")]
    None,
    #[serde(rename = "resource")]
    Resource,
    #[serde(rename = "role")]
    Role,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "user-managed")]
    UserManaged,
}

impl PolicySourceType {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AwsManaged => "aws-managed",
            Self::Group => "group",
            Self::None => "none",
            Self::Resource => "resource",
            Self::Role => "role",
            Self::User => "user",
            Self::UserManaged => "user-managed",
        }
    }
}

impl std::fmt::Display for PolicySourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for PolicySourceType {
    fn from(s: &str) -> Self {
        match s {
            "aws-managed" => Self::AwsManaged,
            "group" => Self::Group,
            "none" => Self::None,
            "resource" => Self::Resource,
            "role" => Self::Role,
            "user" => Self::User,
            "user-managed" => Self::UserManaged,
            _ => Self::default(),
        }
    }
}

/// IAM PolicyUsageType enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PolicyUsageType {
    /// Default variant.
    #[default]
    PermissionsBoundary,
    PermissionsPolicy,
}

impl PolicyUsageType {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PermissionsBoundary => "PermissionsBoundary",
            Self::PermissionsPolicy => "PermissionsPolicy",
        }
    }
}

impl std::fmt::Display for PolicyUsageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for PolicyUsageType {
    fn from(s: &str) -> Self {
        match s {
            "PermissionsBoundary" => Self::PermissionsBoundary,
            "PermissionsPolicy" => Self::PermissionsPolicy,
            _ => Self::default(),
        }
    }
}

/// IAM policyScopeType enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum policyScopeType {
    /// Default variant.
    #[default]
    All,
    #[serde(rename = "AWS")]
    Aws,
    Local,
}

impl policyScopeType {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Aws => "AWS",
            Self::Local => "Local",
        }
    }
}

impl std::fmt::Display for policyScopeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for policyScopeType {
    fn from(s: &str) -> Self {
        match s {
            "All" => Self::All,
            "AWS" => Self::Aws,
            "Local" => Self::Local,
            _ => Self::default(),
        }
    }
}

/// IAM statusType enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum statusType {
    /// Default variant.
    #[default]
    Active,
    Expired,
    Inactive,
}

impl statusType {
    /// Returns the string value of this enum variant.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Expired => "Expired",
            Self::Inactive => "Inactive",
        }
    }
}

impl std::fmt::Display for statusType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for statusType {
    fn from(s: &str) -> Self {
        match s {
            "Active" => Self::Active,
            "Expired" => Self::Expired,
            "Inactive" => Self::Inactive,
            _ => Self::default(),
        }
    }
}

/// IAM AccessKey.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccessKey {
    pub access_key_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_date: Option<chrono::DateTime<chrono::Utc>>,
    pub secret_access_key: String,
    pub status: statusType,
    pub user_name: String,
}

/// IAM AccessKeyLastUsed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccessKeyLastUsed {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_date: Option<chrono::DateTime<chrono::Utc>>,
    pub region: String,
    pub service_name: String,
}

/// IAM AccessKeyMetadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccessKeyMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<statusType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
}

/// IAM AttachedPermissionsBoundary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttachedPermissionsBoundary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions_boundary_arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions_boundary_type: Option<PermissionsBoundaryAttachmentType>,
}

/// IAM AttachedPolicy.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttachedPolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_name: Option<String>,
}

/// IAM ContextEntry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContextEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_key_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_key_type: Option<ContextKeyTypeEnum>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_key_values: Vec<String>,
}

/// IAM DeletionTaskFailureReasonType.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeletionTaskFailureReasonType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub role_usage_list: Vec<RoleUsageType>,
}

/// IAM EvaluationResult.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EvaluationResult {
    pub eval_action_name: String,
    pub eval_decision: PolicyEvaluationDecisionType,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub eval_decision_details: HashMap<String, PolicyEvaluationDecisionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_resource_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matched_statements: Vec<Statement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_context_values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organizations_decision_detail: Option<OrganizationsDecisionDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions_boundary_decision_detail: Option<PermissionsBoundaryDecisionDetail>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resource_specific_results: Vec<ResourceSpecificResult>,
}

/// IAM Group.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Group {
    pub arn: String,
    pub create_date: chrono::DateTime<chrono::Utc>,
    pub group_id: String,
    pub group_name: String,
    pub path: String,
}

/// IAM GroupDetail.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GroupDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arn: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attached_managed_policies: Vec<AttachedPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub group_policy_list: Vec<PolicyDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// IAM InstanceProfile.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InstanceProfile {
    pub arn: String,
    pub create_date: chrono::DateTime<chrono::Utc>,
    pub instance_profile_id: String,
    pub instance_profile_name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<Role>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
}

/// IAM ManagedPolicyDetail.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManagedPolicyDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_version_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_attachable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions_boundary_usage_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_version_list: Vec<PolicyVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_date: Option<chrono::DateTime<chrono::Utc>>,
}

/// IAM OrganizationsDecisionDetail.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OrganizationsDecisionDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_by_organizations: Option<bool>,
}

/// IAM PermissionsBoundaryDecisionDetail.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PermissionsBoundaryDecisionDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_by_permissions_boundary: Option<bool>,
}

/// IAM Policy.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Policy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_version_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_attachable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions_boundary_usage_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_date: Option<chrono::DateTime<chrono::Utc>>,
}

/// IAM PolicyDetail.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PolicyDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_document: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_name: Option<String>,
}

/// IAM PolicyGroup.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PolicyGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
}

/// IAM PolicyRole.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PolicyRole {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_name: Option<String>,
}

/// IAM PolicyUser.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PolicyUser {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
}

/// IAM PolicyVersion.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PolicyVersion {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default_version: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_id: Option<String>,
}

/// IAM Position.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Position {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i32>,
}

/// IAM ResourceSpecificResult.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ResourceSpecificResult {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub eval_decision_details: HashMap<String, PolicyEvaluationDecisionType>,
    pub eval_resource_decision: PolicyEvaluationDecisionType,
    pub eval_resource_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matched_statements: Vec<Statement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_context_values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions_boundary_decision_detail: Option<PermissionsBoundaryDecisionDetail>,
}

/// IAM Role.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Role {
    pub arn: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assume_role_policy_document: Option<String>,
    pub create_date: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_session_duration: Option<i32>,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions_boundary: Option<AttachedPermissionsBoundary>,
    pub role_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_last_used: Option<RoleLastUsed>,
    pub role_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
}

/// IAM RoleDetail.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RoleDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assume_role_policy_document: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attached_managed_policies: Vec<AttachedPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instance_profile_list: Vec<InstanceProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions_boundary: Option<AttachedPermissionsBoundary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_last_used: Option<RoleLastUsed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub role_policy_list: Vec<PolicyDetail>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
}

/// IAM RoleLastUsed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RoleLastUsed {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

/// IAM RoleUsageType.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RoleUsageType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resources: Vec<String>,
}

/// IAM Statement.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Statement {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_position: Option<Position>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_policy_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_policy_type: Option<PolicySourceType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_position: Option<Position>,
}

/// IAM Tag.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Tag {
    pub key: String,
    pub value: String,
}

/// IAM User.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct User {
    pub arn: String,
    pub create_date: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions_boundary: Option<AttachedPermissionsBoundary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    pub user_id: String,
    pub user_name: String,
}

/// IAM UserDetail.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arn: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attached_managed_policies: Vec<AttachedPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub group_list: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions_boundary: Option<AttachedPermissionsBoundary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub user_policy_list: Vec<PolicyDetail>,
}
