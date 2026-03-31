//! Auto-generated from AWS IAM Smithy model. DO NOT EDIT.

use serde::{Deserialize, Serialize};

use crate::types::{ContextEntry, EntityType, PolicyUsageType, Tag, policyScopeType, statusType};

/// IAM AddRoleToInstanceProfileInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AddRoleToInstanceProfileInput {
    pub instance_profile_name: String,
    pub role_name: String,
}

/// IAM AddUserToGroupInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AddUserToGroupInput {
    pub group_name: String,
    pub user_name: String,
}

/// IAM AttachGroupPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttachGroupPolicyInput {
    pub group_name: String,
    pub policy_arn: String,
}

/// IAM AttachRolePolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttachRolePolicyInput {
    pub policy_arn: String,
    pub role_name: String,
}

/// IAM AttachUserPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttachUserPolicyInput {
    pub policy_arn: String,
    pub user_name: String,
}

/// IAM CreateAccessKeyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateAccessKeyInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
}

/// IAM CreateGroupInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateGroupInput {
    pub group_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// IAM CreateInstanceProfileInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateInstanceProfileInput {
    pub instance_profile_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
}

/// IAM CreatePolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreatePolicyInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub policy_document: String,
    pub policy_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
}

/// IAM CreatePolicyVersionInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreatePolicyVersionInput {
    pub policy_arn: String,
    pub policy_document: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_as_default: Option<bool>,
}

/// IAM CreateRoleInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateRoleInput {
    pub assume_role_policy_document: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_session_duration: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions_boundary: Option<String>,
    pub role_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
}

/// IAM CreateServiceLinkedRoleInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateServiceLinkedRoleInput {
    #[serde(rename = "AWSServiceName")]
    pub aws_service_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// IAM CreateUserInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateUserInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions_boundary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    pub user_name: String,
}

/// IAM DeleteAccessKeyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteAccessKeyInput {
    pub access_key_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
}

/// IAM DeleteGroupInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteGroupInput {
    pub group_name: String,
}

/// IAM DeleteGroupPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteGroupPolicyInput {
    pub group_name: String,
    pub policy_name: String,
}

/// IAM DeleteInstanceProfileInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteInstanceProfileInput {
    pub instance_profile_name: String,
}

/// IAM DeletePolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeletePolicyInput {
    pub policy_arn: String,
}

/// IAM DeletePolicyVersionInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeletePolicyVersionInput {
    pub policy_arn: String,
    pub version_id: String,
}

/// IAM DeleteRoleInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteRoleInput {
    pub role_name: String,
}

/// IAM DeleteRolePolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteRolePolicyInput {
    pub policy_name: String,
    pub role_name: String,
}

/// IAM DeleteServiceLinkedRoleInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteServiceLinkedRoleInput {
    pub role_name: String,
}

/// IAM DeleteUserInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteUserInput {
    pub user_name: String,
}

/// IAM DeleteUserPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteUserPolicyInput {
    pub policy_name: String,
    pub user_name: String,
}

/// IAM DetachGroupPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DetachGroupPolicyInput {
    pub group_name: String,
    pub policy_arn: String,
}

/// IAM DetachRolePolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DetachRolePolicyInput {
    pub policy_arn: String,
    pub role_name: String,
}

/// IAM DetachUserPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DetachUserPolicyInput {
    pub policy_arn: String,
    pub user_name: String,
}

/// IAM GetAccessKeyLastUsedInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetAccessKeyLastUsedInput {
    pub access_key_id: String,
}

/// IAM GetAccountAuthorizationDetailsInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetAccountAuthorizationDetailsInput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filter: Vec<EntityType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
}

/// IAM GetGroupInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetGroupInput {
    pub group_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
}

/// IAM GetGroupPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetGroupPolicyInput {
    pub group_name: String,
    pub policy_name: String,
}

/// IAM GetInstanceProfileInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetInstanceProfileInput {
    pub instance_profile_name: String,
}

/// IAM GetPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetPolicyInput {
    pub policy_arn: String,
}

/// IAM GetPolicyVersionInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetPolicyVersionInput {
    pub policy_arn: String,
    pub version_id: String,
}

/// IAM GetRoleInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetRoleInput {
    pub role_name: String,
}

/// IAM GetRolePolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetRolePolicyInput {
    pub policy_name: String,
    pub role_name: String,
}

/// IAM GetServiceLinkedRoleDeletionStatusInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetServiceLinkedRoleDeletionStatusInput {
    pub deletion_task_id: String,
}

/// IAM GetUserInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetUserInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
}

/// IAM GetUserPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetUserPolicyInput {
    pub policy_name: String,
    pub user_name: String,
}

/// IAM ListAccessKeysInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListAccessKeysInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
}

/// IAM ListAttachedGroupPoliciesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListAttachedGroupPoliciesInput {
    pub group_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,
}

/// IAM ListAttachedRolePoliciesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListAttachedRolePoliciesInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,
    pub role_name: String,
}

/// IAM ListAttachedUserPoliciesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListAttachedUserPoliciesInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,
    pub user_name: String,
}

/// IAM ListEntitiesForPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListEntitiesForPolicyInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_filter: Option<EntityType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,
    pub policy_arn: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_usage_filter: Option<PolicyUsageType>,
}

/// IAM ListGroupPoliciesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListGroupPoliciesInput {
    pub group_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
}

/// IAM ListGroupsForUserInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListGroupsForUserInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    pub user_name: String,
}

/// IAM ListGroupsInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListGroupsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,
}

/// IAM ListInstanceProfilesForRoleInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListInstanceProfilesForRoleInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    pub role_name: String,
}

/// IAM ListInstanceProfilesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListInstanceProfilesInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,
}

/// IAM ListPoliciesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListPoliciesInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_attached: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_usage_filter: Option<PolicyUsageType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<policyScopeType>,
}

/// IAM ListPolicyVersionsInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListPolicyVersionsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    pub policy_arn: String,
}

/// IAM ListRolePoliciesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListRolePoliciesInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    pub role_name: String,
}

/// IAM ListRoleTagsInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListRoleTagsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    pub role_name: String,
}

/// IAM ListRolesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListRolesInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,
}

/// IAM ListUserPoliciesInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListUserPoliciesInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    pub user_name: String,
}

/// IAM ListUserTagsInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListUserTagsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    pub user_name: String,
}

/// IAM ListUsersInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListUsersInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,
}

/// IAM PutGroupPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PutGroupPolicyInput {
    pub group_name: String,
    pub policy_document: String,
    pub policy_name: String,
}

/// IAM PutRolePolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PutRolePolicyInput {
    pub policy_document: String,
    pub policy_name: String,
    pub role_name: String,
}

/// IAM PutUserPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PutUserPolicyInput {
    pub policy_document: String,
    pub policy_name: String,
    pub user_name: String,
}

/// IAM RemoveRoleFromInstanceProfileInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RemoveRoleFromInstanceProfileInput {
    pub instance_profile_name: String,
    pub role_name: String,
}

/// IAM RemoveUserFromGroupInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RemoveUserFromGroupInput {
    pub group_name: String,
    pub user_name: String,
}

/// IAM SetDefaultPolicyVersionInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetDefaultPolicyVersionInput {
    pub policy_arn: String,
    pub version_id: String,
}

/// IAM SimulateCustomPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SimulateCustomPolicyInput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub action_names: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller_arn: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_entries: Vec<ContextEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions_boundary_policy_input_list: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_input_list: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resource_arns: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_handling_option: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_policy: Option<String>,
}

/// IAM SimulatePrincipalPolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SimulatePrincipalPolicyInput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub action_names: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller_arn: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_entries: Vec<ContextEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions_boundary_policy_input_list: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_input_list: Vec<String>,
    pub policy_source_arn: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resource_arns: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_handling_option: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_policy: Option<String>,
}

/// IAM TagRoleInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TagRoleInput {
    pub role_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
}

/// IAM TagUserInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TagUserInput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    pub user_name: String,
}

/// IAM UntagRoleInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UntagRoleInput {
    pub role_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tag_keys: Vec<String>,
}

/// IAM UntagUserInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UntagUserInput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tag_keys: Vec<String>,
    pub user_name: String,
}

/// IAM UpdateAccessKeyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateAccessKeyInput {
    pub access_key_id: String,
    pub status: statusType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
}

/// IAM UpdateAssumeRolePolicyInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateAssumeRolePolicyInput {
    pub policy_document: String,
    pub role_name: String,
}

/// IAM UpdateGroupInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateGroupInput {
    pub group_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_group_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_path: Option<String>,
}

/// IAM UpdateRoleInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateRoleInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_session_duration: Option<i32>,
    pub role_name: String,
}

/// IAM UpdateUserInput.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateUserInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_user_name: Option<String>,
    pub user_name: String,
}
