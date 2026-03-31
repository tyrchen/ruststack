//! Auto-generated from AWS IAM Smithy model. DO NOT EDIT.

use serde::{Deserialize, Serialize};

use crate::types::{
    AccessKey, AccessKeyLastUsed, AccessKeyMetadata, AttachedPolicy, DeletionTaskFailureReasonType,
    DeletionTaskStatusType, EvaluationResult, Group, GroupDetail, InstanceProfile,
    ManagedPolicyDetail, Policy, PolicyGroup, PolicyRole, PolicyUser, PolicyVersion, Role,
    RoleDetail, Tag, User, UserDetail,
};

/// IAM CreateAccessKeyResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateAccessKeyResponse {
    pub access_key: AccessKey,
}

/// IAM CreateGroupResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateGroupResponse {
    pub group: Group,
}

/// IAM CreateInstanceProfileResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateInstanceProfileResponse {
    pub instance_profile: InstanceProfile,
}

/// IAM CreatePolicyResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreatePolicyResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<Policy>,
}

/// IAM CreatePolicyVersionResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreatePolicyVersionResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_version: Option<PolicyVersion>,
}

/// IAM CreateRoleResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateRoleResponse {
    pub role: Role,
}

/// IAM CreateServiceLinkedRoleResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateServiceLinkedRoleResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
}

/// IAM CreateUserResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateUserResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
}

/// IAM DeleteServiceLinkedRoleResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteServiceLinkedRoleResponse {
    pub deletion_task_id: String,
}

/// IAM GetAccessKeyLastUsedResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetAccessKeyLastUsedResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key_last_used: Option<AccessKeyLastUsed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
}

/// IAM GetAccountAuthorizationDetailsResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetAccountAuthorizationDetailsResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub group_detail_list: Vec<GroupDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policies: Vec<ManagedPolicyDetail>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub role_detail_list: Vec<RoleDetail>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub user_detail_list: Vec<UserDetail>,
}

/// IAM GetGroupPolicyResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetGroupPolicyResponse {
    pub group_name: String,
    pub policy_document: String,
    pub policy_name: String,
}

/// IAM GetGroupResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetGroupResponse {
    pub group: Group,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<User>,
}

/// IAM GetInstanceProfileResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetInstanceProfileResponse {
    pub instance_profile: InstanceProfile,
}

/// IAM GetPolicyResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetPolicyResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<Policy>,
}

/// IAM GetPolicyVersionResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetPolicyVersionResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_version: Option<PolicyVersion>,
}

/// IAM GetRolePolicyResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetRolePolicyResponse {
    pub policy_document: String,
    pub policy_name: String,
    pub role_name: String,
}

/// IAM GetRoleResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetRoleResponse {
    pub role: Role,
}

/// IAM GetServiceLinkedRoleDeletionStatusResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetServiceLinkedRoleDeletionStatusResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<DeletionTaskFailureReasonType>,
    pub status: DeletionTaskStatusType,
}

/// IAM GetUserPolicyResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetUserPolicyResponse {
    pub policy_document: String,
    pub policy_name: String,
    pub user_name: String,
}

/// IAM GetUserResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetUserResponse {
    pub user: User,
}

/// IAM ListAccessKeysResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListAccessKeysResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub access_key_metadata: Vec<AccessKeyMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// IAM ListAttachedGroupPoliciesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListAttachedGroupPoliciesResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attached_policies: Vec<AttachedPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// IAM ListAttachedRolePoliciesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListAttachedRolePoliciesResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attached_policies: Vec<AttachedPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// IAM ListAttachedUserPoliciesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListAttachedUserPoliciesResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attached_policies: Vec<AttachedPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// IAM ListEntitiesForPolicyResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListEntitiesForPolicyResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_groups: Vec<PolicyGroup>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_roles: Vec<PolicyRole>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_users: Vec<PolicyUser>,
}

/// IAM ListGroupPoliciesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListGroupPoliciesResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_names: Vec<String>,
}

/// IAM ListGroupsForUserResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListGroupsForUserResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<Group>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// IAM ListGroupsResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListGroupsResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<Group>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// IAM ListInstanceProfilesForRoleResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListInstanceProfilesForRoleResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instance_profiles: Vec<InstanceProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// IAM ListInstanceProfilesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListInstanceProfilesResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instance_profiles: Vec<InstanceProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// IAM ListPoliciesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListPoliciesResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policies: Vec<Policy>,
}

/// IAM ListPolicyVersionsResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListPolicyVersionsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub versions: Vec<PolicyVersion>,
}

/// IAM ListRolePoliciesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListRolePoliciesResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_names: Vec<String>,
}

/// IAM ListRoleTagsResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListRoleTagsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
}

/// IAM ListRolesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListRolesResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<Role>,
}

/// IAM ListUserPoliciesResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListUserPoliciesResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_names: Vec<String>,
}

/// IAM ListUserTagsResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListUserTagsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
}

/// IAM ListUsersResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListUsersResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<User>,
}

/// IAM SimulatePolicyResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SimulatePolicyResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evaluation_results: Vec<EvaluationResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// IAM UpdateRoleResponse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateRoleResponse {}
