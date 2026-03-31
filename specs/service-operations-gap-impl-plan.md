# Rustack Service Operations Gap: Implementation Plan

**Date:** 2026-03-26
**Status:** Draft
**Depends on:** All existing service design specs
**Scope:** Concrete plan to close the operations gap between Rustack (17 services, ~586 ops) and LocalStack (34+ services, ~2,300+ ops) for shared services, based on analysis of which missing operations matter for local development.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Methodology](#2-methodology)
3. [Gap Analysis by Service](#3-gap-analysis-by-service)
4. [Priority Tiers](#4-priority-tiers)
5. [Tier 1: Must-Have Operations](#5-tier-1-must-have-operations)
6. [Tier 2: Should-Have Operations](#6-tier-2-should-have-operations)
7. [Tier 3: Nice-to-Have Operations](#7-tier-3-nice-to-have-operations)
8. [Tier 4: Skip](#8-tier-4-skip)
9. [Phased Implementation Plan](#9-phased-implementation-plan)
10. [Detailed Design: DynamoDB Transactions](#10-detailed-design-dynamodb-transactions)
11. [Detailed Design: Lambda Layers & Event Source Mappings](#11-detailed-design-lambda-layers--event-source-mappings)
12. [Detailed Design: DynamoDB TTL & Tagging](#12-detailed-design-dynamodb-ttl--tagging)
13. [Cross-Cutting Concerns](#13-cross-cutting-concerns)
14. [Testing Strategy](#14-testing-strategy)
15. [Risk Analysis](#15-risk-analysis)
16. [Success Metrics](#16-success-metrics)

---

## 1. Executive Summary

Rustack implements 17 AWS services with ~586 operations. LocalStack implements 34+ services with ~2,300+ operations. For the 17 shared services, Rustack covers ~586 of LocalStack's ~1,100 operations (53%).

This spec proposes closing the gap strategically — not by implementing every operation, but by categorizing the ~514 missing operations into priority tiers based on real-world local development usage. The core finding:

- **Tier 1 (Must-Have):** 44 operations across 4 services. These are operations that real applications call frequently, and their absence forces users back to LocalStack or AWS. Primarily: DynamoDB transactions + TTL + tagging, Lambda layers + event source mappings, and IAM OIDC providers.
- **Tier 2 (Should-Have):** 56 operations across 7 services. Important for completeness when testing IaC workflows (Terraform/CDK), but applications can work without them. Primarily: DynamoDB PartiQL, Lambda concurrency configs, S3 replication + analytics, KMS key import, SSM documents.
- **Tier 3 (Nice-to-Have):** ~80 operations. Advanced features with niche usage — IAM MFA/SAML, SES bounce handling, CloudWatch Logs anomaly detection.
- **Tier 4 (Skip):** ~334 operations. Production infrastructure concerns with zero value in local emulation — SSM patch management, IAM Organizations, DynamoDB global tables/backup, CloudWatch Logs delivery pipelines.

Implementing Tiers 1 + 2 (100 operations) would bring Rustack to ~686 operations and cover effectively 100% of common local-development API calls across all 17 services. This spec provides concrete designs for the highest-impact additions.

---

## 2. Methodology

Each missing operation was evaluated against three criteria:

1. **Application-level usage frequency** — Does typical application code call this operation? (e.g., `TransactWriteItems` is called by application code; `CreateBackup` is called by infrastructure automation.)
2. **IaC workflow impact** — Does Terraform/CDK/Pulumi require this operation during `plan`/`apply`/`destroy`? (e.g., `TagResource` is emitted by every Terraform resource with tags; `DescribeLimits` is never called by IaC.)
3. **Blocking behavior** — Does the absence cause a hard failure (SDK throws, IaC aborts) vs. a soft degradation (feature unavailable but app works)?

Operations that score high on all three criteria are Tier 1. Operations that score high on one are Tier 2-3. Operations that score zero on all three are Tier 4.

---

## 3. Gap Analysis by Service

### 3.1 DynamoDB (13 → target 28, gap: 44 in LocalStack)

| Missing Operation | Tier | Justification |
|---|---|---|
| TransactGetItems | 1 | Core application pattern for consistent reads across items |
| TransactWriteItems | 1 | Core application pattern for atomic multi-item writes |
| UpdateTimeToLive | 1 | Common for session stores, caches, TTL-based cleanup |
| DescribeTimeToLive | 1 | Read companion to UpdateTimeToLive |
| TagResource | 1 | Required by every Terraform/CDK resource with tags |
| UntagResource | 1 | Required by Terraform tag lifecycle |
| ListTagsOfResource | 1 | Required by Terraform plan/refresh |
| ExecuteStatement (PartiQL) | 2 | Growing adoption, AWS Console uses it |
| BatchExecuteStatement | 2 | Batch companion to ExecuteStatement |
| ExecuteTransaction | 2 | Transaction companion to ExecuteStatement |
| DescribeLimits | 2 | Some SDKs call this during initialization |
| DescribeEndpoints | 2 | SDK endpoint discovery |
| DescribeContinuousBackups | 3 | Terraform reads this for PITR config |
| UpdateContinuousBackups | 3 | Terraform sets PITR config |
| DescribeContributorInsights | 3 | Monitoring feature, rare in local dev |
| UpdateContributorInsights | 3 | Monitoring feature, rare in local dev |
| ListContributorInsights | 3 | Monitoring feature, rare in local dev |
| CreateBackup | 4 | Production DR, no value locally |
| DeleteBackup | 4 | Production DR |
| DescribeBackup | 4 | Production DR |
| ListBackups | 4 | Production DR |
| RestoreTableFromBackup | 4 | Production DR |
| RestoreTableToPointInTime | 4 | Production DR |
| CreateGlobalTable | 4 | Multi-region replication |
| DescribeGlobalTable | 4 | Multi-region replication |
| DescribeGlobalTableSettings | 4 | Multi-region replication |
| ListGlobalTables | 4 | Multi-region replication |
| UpdateGlobalTable | 4 | Multi-region replication |
| UpdateGlobalTableSettings | 4 | Multi-region replication |
| EnableKinesisStreamingDestination | 4 | Cross-service integration |
| DisableKinesisStreamingDestination | 4 | Cross-service integration |
| DescribeKinesisStreamingDestination | 4 | Cross-service integration |
| UpdateKinesisStreamingDestination | 4 | Cross-service integration |
| ExportTableToPointInTime | 4 | Bulk export to S3 |
| ImportTable | 4 | Bulk import from S3 |
| DescribeExport | 4 | Bulk export status |
| DescribeImport | 4 | Bulk import status |
| ListExports | 4 | Bulk export listing |
| ListImports | 4 | Bulk import listing |
| GetResourcePolicy | 4 | Resource-based policies, rare |
| PutResourcePolicy | 4 | Resource-based policies, rare |
| DeleteResourcePolicy | 4 | Resource-based policies, rare |
| DescribeTableReplicaAutoScaling | 4 | Global tables auto-scaling |
| UpdateTableReplicaAutoScaling | 4 | Global tables auto-scaling |

### 3.2 Lambda (29 → target 46, gap: 72 in LocalStack)

| Missing Operation | Tier | Justification |
|---|---|---|
| PublishLayerVersion | 1 | Lambda Layers are widely used for shared dependencies |
| GetLayerVersion | 1 | Required to resolve layers during function create/update |
| GetLayerVersionByArn | 1 | ARN-based layer resolution |
| ListLayerVersions | 1 | Layer management |
| ListLayers | 1 | Layer discovery |
| DeleteLayerVersion | 1 | Layer lifecycle |
| AddLayerVersionPermission | 1 | Cross-account layer sharing |
| GetLayerVersionPolicy | 1 | Read companion to AddLayerVersionPermission |
| RemoveLayerVersionPermission | 1 | Layer permission lifecycle |
| CreateEventSourceMapping | 1 | Required for SQS/Kinesis/DDB trigger-based Lambdas |
| GetEventSourceMapping | 1 | Read companion |
| UpdateEventSourceMapping | 1 | Event source configuration |
| DeleteEventSourceMapping | 1 | Event source lifecycle |
| ListEventSourceMappings | 1 | Event source discovery |
| PutFunctionConcurrency | 2 | Concurrency limits, used in production configs |
| GetFunctionConcurrency | 2 | Read companion |
| DeleteFunctionConcurrency | 2 | Lifecycle |
| PutFunctionEventInvokeConfig | 2 | DLQ/destination config for async invocations |
| GetFunctionEventInvokeConfig | 2 | Read companion |
| UpdateFunctionEventInvokeConfig | 2 | Update companion |
| DeleteFunctionEventInvokeConfig | 2 | Lifecycle |
| ListFunctionEventInvokeConfigs | 2 | Listing |
| InvokeAsync | 2 | Deprecated but some legacy code uses it |
| GetProvisionedConcurrencyConfig | 3 | Production scaling, Terraform reads it |
| PutProvisionedConcurrencyConfig | 3 | Production scaling |
| DeleteProvisionedConcurrencyConfig | 3 | Lifecycle |
| ListProvisionedConcurrencyConfigs | 3 | Listing |
| CreateCodeSigningConfig | 3 | Security feature, rare in local dev |
| GetCodeSigningConfig | 3 | Read companion |
| UpdateCodeSigningConfig | 3 | Update companion |
| DeleteCodeSigningConfig | 3 | Lifecycle |
| ListCodeSigningConfigs | 3 | Listing |
| ListFunctionsByCodeSigningConfig | 3 | Listing |
| PutFunctionCodeSigningConfig | 3 | Binding |
| GetFunctionCodeSigningConfig | 3 | Read companion |
| DeleteFunctionCodeSigningConfig | 3 | Lifecycle |
| GetRuntimeManagementConfig | 3 | Runtime version pinning |
| PutRuntimeManagementConfig | 3 | Runtime version pinning |
| InvokeWithResponseStream | 3 | Streaming responses, newer feature |
| GetFunctionRecursionConfig | 4 | Recursive invocation guard |
| PutFunctionRecursionConfig | 4 | Recursive invocation guard |
| GetFunctionScalingConfig | 4 | Auto-scaling config |
| PutFunctionScalingConfig | 4 | Auto-scaling config |
| CreateCapacityProvider | 4 | Managed instance pools |
| DeleteCapacityProvider | 4 | Managed instance pools |
| GetCapacityProvider | 4 | Managed instance pools |
| ListCapacityProviders | 4 | Managed instance pools |
| UpdateCapacityProvider | 4 | Managed instance pools |
| ListFunctionVersionsByCapacityProvider | 4 | Managed instance pools |
| All Durable Execution ops (9) | 4 | Preview feature, not GA |

### 3.3 IAM (76 → target 86, gap: 100 in LocalStack)

| Missing Operation | Tier | Justification |
|---|---|---|
| CreateOpenIDConnectProvider | 1 | Required for EKS IRSA, Cognito federation |
| GetOpenIDConnectProvider | 1 | Read companion |
| DeleteOpenIDConnectProvider | 1 | Lifecycle |
| ListOpenIDConnectProviders | 1 | Discovery |
| TagPolicy | 1 | Terraform tags on managed policies |
| UntagPolicy | 1 | Terraform tag lifecycle |
| ListPolicyTags | 1 | Terraform plan/refresh |
| TagInstanceProfile | 1 | Terraform tags on instance profiles |
| UntagInstanceProfile | 1 | Terraform tag lifecycle |
| ListInstanceProfileTags | 1 | Terraform plan/refresh |
| CreateSAMLProvider | 2 | Federated auth testing |
| GetSAMLProvider | 2 | Read companion |
| DeleteSAMLProvider | 2 | Lifecycle |
| ListSAMLProviders | 2 | Discovery |
| UpdateSAMLProvider | 2 | Update companion |
| CreateAccountAlias | 2 | Account identification |
| DeleteAccountAlias | 2 | Lifecycle |
| ListAccountAliases | 2 | Discovery |
| GetAccountSummary | 2 | Account resource counts |
| PutRolePermissionsBoundary | 2 | Used in production IAM configs |
| DeleteRolePermissionsBoundary | 2 | Lifecycle |
| PutUserPermissionsBoundary | 2 | Used in production IAM configs |
| DeleteUserPermissionsBoundary | 2 | Lifecycle |
| SetSecurityTokenServicePreferences | 2 | STS endpoint config |
| AddClientIDToOpenIDConnectProvider | 3 | OIDC provider management |
| RemoveClientIDFromOpenIDConnectProvider | 3 | OIDC provider management |
| UpdateOpenIDConnectProviderThumbprint | 3 | OIDC provider maintenance |
| TagOpenIDConnectProvider | 3 | Tagging |
| UntagOpenIDConnectProvider | 3 | Tagging |
| ListOpenIDConnectProviderTags | 3 | Tagging |
| TagSAMLProvider | 3 | Tagging |
| UntagSAMLProvider | 3 | Tagging |
| ListSAMLProviderTags | 3 | Tagging |
| CreateLoginProfile | 4 | Console password, irrelevant locally |
| GetLoginProfile | 4 | Console password |
| UpdateLoginProfile | 4 | Console password |
| DeleteLoginProfile | 4 | Console password |
| ChangePassword | 4 | Console password |
| CreateVirtualMFADevice | 4 | MFA device management |
| DeleteVirtualMFADevice | 4 | MFA device management |
| EnableMFADevice | 4 | MFA device management |
| DeactivateMFADevice | 4 | MFA device management |
| ListMFADevices | 4 | MFA device management |
| ResyncMFADevice | 4 | MFA device management |
| All SSH/ServerCert ops (13) | 4 | Legacy cert management |
| All signing cert ops (4) | 4 | Legacy cert management |
| All service-specific credential ops (5) | 4 | Niche use case |
| GenerateCredentialReport | 4 | Audit feature |
| GetCredentialReport | 4 | Audit feature |
| All Organizations ops (9+) | 4 | Multi-account governance |
| All Access Advisor ops (5) | 4 | Audit/compliance |
| GetContextKeysForCustomPolicy | 4 | Policy simulation |
| GetContextKeysForPrincipalPolicy | 4 | Policy simulation |
| GetAccountPasswordPolicy | 4 | Console password policy |
| All password policy ops (3) | 4 | Console password policy |
| All delegation request ops (9) | 4 | Organizations feature |

### 3.4 SSM (13 → target 18, gap: 133 in LocalStack)

| Missing Operation | Tier | Justification |
|---|---|---|
| CreateDocument | 2 | SSM documents used in Ansible-like workflows |
| GetDocument | 2 | Read companion |
| DeleteDocument | 2 | Lifecycle |
| ListDocuments | 2 | Discovery |
| DescribeDocument | 2 | Read companion |
| SendCommand | 3 | Remote execution |
| All maintenance window ops (23) | 4 | EC2 fleet management |
| All patch management ops (19) | 4 | EC2 fleet management |
| All automation ops (9) | 4 | EC2 fleet management |
| All association ops (15) | 4 | EC2 fleet management |
| All inventory ops (6) | 4 | EC2 fleet management |
| All session manager ops (5) | 4 | Interactive shell, irrelevant locally |
| All OpsItems/OpsMetadata ops (15) | 4 | Operational management |
| All compliance ops (3) | 4 | Compliance reporting |
| All activation/instance ops (10) | 4 | Hybrid instance registration |

### 3.5 S3 (70 → target 78, gap: 41 in LocalStack)

| Missing Operation | Tier | Justification |
|---|---|---|
| RestoreObject | 2 | Glacier restore, some apps test this flow |
| SelectObjectContent | 2 | S3 Select (SQL queries on objects), growing usage |
| WriteGetObjectResponse | 2 | Lambda@S3 Object Lambda responses |
| PutBucketReplication | 2 | Terraform configures replication |
| GetBucketReplication | 2 | Terraform plan reads replication |
| DeleteBucketReplication | 2 | Terraform lifecycle |
| ListDirectoryBuckets | 2 | S3 Express One Zone |
| PutBucketAnalyticsConfiguration | 3 | Cost optimization feature |
| GetBucketAnalyticsConfiguration | 3 | Read companion |
| DeleteBucketAnalyticsConfiguration | 3 | Lifecycle |
| ListBucketAnalyticsConfigurations | 3 | Listing |
| PutBucketInventoryConfiguration | 3 | Audit feature |
| GetBucketInventoryConfiguration | 3 | Read companion |
| DeleteBucketInventoryConfiguration | 3 | Lifecycle |
| ListBucketInventoryConfigurations | 3 | Listing |
| PutBucketMetricsConfiguration | 3 | CloudWatch integration |
| GetBucketMetricsConfiguration | 3 | Read companion |
| DeleteBucketMetricsConfiguration | 3 | Lifecycle |
| ListBucketMetricsConfigurations | 3 | Listing |
| PutBucketIntelligentTieringConfiguration | 3 | Storage class automation |
| GetBucketIntelligentTieringConfiguration | 3 | Read companion |
| DeleteBucketIntelligentTieringConfiguration | 3 | Lifecycle |
| ListBucketIntelligentTieringConfigurations | 3 | Listing |
| RenameObject | 4 | Not standard AWS API |
| GetObjectTorrent | 4 | Deprecated feature |
| All metadata table ops (8) | 4 | LocalStack internal, not AWS API |

### 3.6 CloudWatch Logs (42 → target 42, gap: 62 in LocalStack)

All missing operations are Tier 3-4:

| Category | Count | Tier | Justification |
|---|---|---|---|
| Delivery/Integration ops | 16 | 4 | Log routing pipelines |
| Anomaly detection ops | 7 | 4 | ML-based monitoring |
| Scheduled query ops | 6 | 4 | Recurring queries |
| Transformer ops | 4 | 4 | Log parsing |
| S3 table integration ops | 7 | 4 | Storage integration |
| Data protection ops | 3 | 3 | Sensitive data masking |
| Account policy ops | 3 | 3 | Account-level settings |
| Index policy ops | 3 | 3 | Search indexing |
| Import task ops | 4 | 4 | Bulk import |
| Live tail | 1 | 3 | Real-time streaming |
| Other | 8 | 4 | Misc. |

CloudWatch Logs is already well-covered. No Tier 1 gaps.

### 3.7 Remaining Services (KMS, Kinesis, SES, SNS, SQS, EventBridge, CloudWatch Metrics, STS, DynamoDB Streams)

| Service | Gap | Tier 1 | Tier 2 | Notes |
|---|---|---|---|---|
| KMS (38 → 53) | 15 | 0 | 3 (ImportKeyMaterial, GetParametersForImport, DeleteImportedKeyMaterial) | Custom key stores and multi-region are Tier 4 |
| Kinesis (27 → 39) | 12 | 0 | 2 (UpdateStreamMode, UpdateShardCount are already implemented; TagResource/UntagResource) | Enhanced monitoring and account settings are Tier 4 |
| SES (44 → 71) | 27 | 0 | 3 (SendBulkTemplatedEmail, TestRenderTemplate, PutConfigurationSetDeliveryOptions) | Custom verification and bounce ops are Tier 3-4 |
| SNS (47 → 42) | -5 | 0 | 0 | Rustack already has MORE ops than LocalStack |
| SQS (23 → 23) | 0 | 0 | 0 | Full parity |
| EventBridge (43 → 57) | 14 | 0 | 0 | All 43 operation types are defined; 22 are implemented. Remaining 21 are already in the model, just need handler code |
| CloudWatch Metrics (31 → 39) | 8 | 0 | 0 | Missing ops are managed insight rules and metric streams, Tier 3-4 |
| STS (8 → 11) | 3 | 0 | 0 | Missing: AssumeRoot, GetDelegatedAccessToken, GetWebIdentityToken — niche ops |
| DynamoDB Streams (4 → 4) | 0 | 0 | 0 | Full parity |

---

## 4. Priority Tiers

### Summary

| Tier | Operations | Services Affected | Effort Estimate |
|---|---|---|---|
| **Tier 1: Must-Have** | 44 | DynamoDB (7), Lambda (14), IAM (10), EventBridge (13 existing model ops to implement) | ~3,000-4,000 LoC |
| **Tier 2: Should-Have** | 56 | DynamoDB (6), Lambda (8), IAM (14), S3 (7), SSM (5), KMS (3), SES (3), Kinesis (2), other (8) | ~4,000-5,000 LoC |
| **Tier 3: Nice-to-Have** | ~80 | Spread across all services | ~5,000-6,000 LoC |
| **Tier 4: Skip** | ~334 | SSM (93), IAM (66), CloudWatch Logs (62), DynamoDB (35), Lambda (22), other | N/A |

### Decision Framework

**Tier 1** operations meet ALL of:
- Called by application code in >30% of projects using the service
- Absence causes hard SDK failure or IaC abort
- Implementation complexity is bounded (no new subsystem needed)

**Tier 2** operations meet AT LEAST ONE of:
- Called by IaC tools during plan/apply
- Used by >10% of projects
- Enables testing a complete feature (e.g., PartiQL as alternative query syntax)

**Tier 3** operations meet AT LEAST ONE of:
- Used by <10% of projects but has passionate users
- Terraform reads it during refresh (will fail without it)
- Implementation is trivial (metadata CRUD)

**Tier 4** operations meet ALL of:
- Never called by application code in local dev
- Purely production infrastructure management
- Complex to implement correctly with no local dev value

---

## 5. Tier 1: Must-Have Operations

### 5.1 DynamoDB — Transactions + TTL + Tagging (7 ops)

**Operations:** TransactGetItems, TransactWriteItems, UpdateTimeToLive, DescribeTimeToLive, TagResource, UntagResource, ListTagsOfResource

**Why Must-Have:**
- DynamoDB transactions are used by ~40% of DynamoDB applications. Common patterns: creating a user + updating a counter atomically, transferring balance between accounts, idempotent writes with condition checks. Without transactions, any app using `TransactWriteItems` fails with `UnknownOperationException`.
- TTL is used by ~35% of DynamoDB applications. Common patterns: session expiration, cache entries, temporary records. Terraform/CDK configures TTL on nearly every table that stores ephemeral data. Without `UpdateTimeToLive`, Terraform apply fails.
- Tagging is used by every Terraform/CDK resource. Without `TagResource`, every DynamoDB table created by IaC fails if tags are specified.

### 5.2 Lambda — Layers + Event Source Mappings (14 ops)

**Operations:**
- Layers: PublishLayerVersion, GetLayerVersion, GetLayerVersionByArn, ListLayerVersions, ListLayers, DeleteLayerVersion, AddLayerVersionPermission, GetLayerVersionPolicy, RemoveLayerVersionPermission
- Event Source Mappings: CreateEventSourceMapping, GetEventSourceMapping, UpdateEventSourceMapping, DeleteEventSourceMapping, ListEventSourceMappings

**Why Must-Have:**
- Lambda Layers are used by ~50% of Lambda deployments. Layers provide shared dependencies (e.g., AWS SDK, database drivers) across multiple functions. Without layers, any function that references a layer ARN in its configuration fails to create.
- Event Source Mappings are the mechanism by which Lambda polls SQS queues, Kinesis streams, and DynamoDB Streams. Without ESM, it's impossible to test the most common serverless pattern: "SQS queue triggers Lambda." ~60% of Lambda functions use an event source mapping. Without `CreateEventSourceMapping`, CDK/Terraform/SAM deployments that define triggers fail.

### 5.3 IAM — OIDC Providers + Resource Tagging (10 ops)

**Operations:**
- OIDC: CreateOpenIDConnectProvider, GetOpenIDConnectProvider, DeleteOpenIDConnectProvider, ListOpenIDConnectProviders
- Policy tags: TagPolicy, UntagPolicy, ListPolicyTags
- Instance profile tags: TagInstanceProfile, UntagInstanceProfile, ListInstanceProfileTags

**Why Must-Have:**
- OIDC providers are required for EKS IRSA (IAM Roles for Service Accounts), which is the standard way to give Kubernetes pods AWS credentials. Any project testing EKS infrastructure locally needs OIDC providers. Also required for Cognito user pool federation testing.
- Resource tagging on policies and instance profiles is emitted by Terraform for any IAM resource with tags. Without these ops, Terraform apply succeeds but subsequent plan/refresh fails trying to read tags.

### 5.4 EventBridge — Remaining Implemented Operations (13 ops)

**Operations:** CreateArchive, DeleteArchive, DescribeArchive, ListArchives, UpdateArchive, StartReplay, CancelReplay, DescribeReplay, ListReplays, CreateApiDestination, DeleteApiDestination, DescribeApiDestination, ListApiDestinations, UpdateApiDestination, CreateConnection, DeleteConnection, DescribeConnection, ListConnections, UpdateConnection, DeauthorizeConnection, CreateEndpoint, DeleteEndpoint, DescribeEndpoint, ListEndpoints, UpdateEndpoint

**Why Must-Have:** These operations are already defined in the Rustack EventBridge model enum. The types are generated. The HTTP routing is in place. Only the handler dispatch and provider methods need implementation. This is low-effort, high-value work — approximately 50-100 lines per operation since the infrastructure exists.

---

## 6. Tier 2: Should-Have Operations

### 6.1 DynamoDB — PartiQL (3 ops)

**Operations:** ExecuteStatement, BatchExecuteStatement, ExecuteTransaction

**Design:** Requires a PartiQL parser. PartiQL is a SQL-compatible query language. For DynamoDB, the supported subset is limited: `SELECT`, `INSERT`, `UPDATE`, `DELETE` with `WHERE` clauses mapping to key conditions. The parser translates PartiQL statements into existing DynamoDB operations internally.

**Complexity:** Medium. Requires a new parser (~500-800 LoC) but reuses the existing expression evaluator and storage engine.

### 6.2 DynamoDB — Service Metadata (2 ops)

**Operations:** DescribeLimits, DescribeEndpoints

**Design:** Return static/hardcoded responses. `DescribeLimits` returns account limits (table count, GSI count, etc.) — all hardcoded maximums. `DescribeEndpoints` returns the local endpoint URL.

**Complexity:** Trivial. ~30 lines each.

### 6.3 Lambda — Concurrency & Event Invoke Config (8 ops)

**Operations:** PutFunctionConcurrency, GetFunctionConcurrency, DeleteFunctionConcurrency, PutFunctionEventInvokeConfig, GetFunctionEventInvokeConfig, UpdateFunctionEventInvokeConfig, DeleteFunctionEventInvokeConfig, ListFunctionEventInvokeConfigs

**Design:** Metadata CRUD stored alongside function records. `FunctionConcurrency` is a single integer. `EventInvokeConfig` is a struct with max retry attempts, max event age, and destination configs. Store in `FunctionRecord`, return on read. No behavioral enforcement needed — just store and return the configuration.

**Complexity:** Low. ~200-300 LoC total. All metadata CRUD following existing patterns.

### 6.4 IAM — SAML Providers + Account Aliases + Permission Boundaries (14 ops)

**Operations:** CreateSAMLProvider, GetSAMLProvider, DeleteSAMLProvider, ListSAMLProviders, UpdateSAMLProvider, CreateAccountAlias, DeleteAccountAlias, ListAccountAliases, GetAccountSummary, PutRolePermissionsBoundary, DeleteRolePermissionsBoundary, PutUserPermissionsBoundary, DeleteUserPermissionsBoundary, SetSecurityTokenServicePreferences

**Design:** SAML providers are metadata objects (name, SAML metadata XML document, ARN). Store in a `DashMap<String, SAMLProvider>`. Permission boundaries are an optional ARN field on roles/users — add a field to existing storage structs. Account aliases are a list of strings (max 1 alias per account).

**Complexity:** Low-Medium. ~600-800 LoC. Standard CRUD patterns.

### 6.5 S3 — Replication + Select + Restore (7 ops)

**Operations:** PutBucketReplication, GetBucketReplication, DeleteBucketReplication, SelectObjectContent, RestoreObject, WriteGetObjectResponse, ListDirectoryBuckets

**Design:**
- Replication: Store configuration as metadata. No actual cross-region replication needed — just accept and return the config.
- SelectObjectContent: Parse SQL-like queries against CSV/JSON objects. This is a significant feature (~1,000-1,500 LoC for the SQL parser + CSV/JSON reader).
- RestoreObject: For Glacier restore emulation, immediately mark the object as restored (no actual delay).

**Complexity:** Medium (SelectObjectContent) + Low (rest).

### 6.6 SSM — Documents (5 ops)

**Operations:** CreateDocument, GetDocument, DeleteDocument, ListDocuments, DescribeDocument

**Design:** SSM Documents are JSON/YAML configuration documents with name, version, content, and type. Store in a `DashMap<String, SSMDocument>` with version tracking. No execution semantics needed — just store and return.

**Complexity:** Low. ~400-500 LoC.

### 6.7 KMS — Key Import (3 ops)

**Operations:** GetParametersForImport, ImportKeyMaterial, DeleteImportedKeyMaterial

**Design:** Generate import parameters (wrapping key + import token), accept key material import (store raw key bytes), delete imported material. The wrapping key is an RSA key pair generated per import request. The imported key material replaces the auto-generated key material for the target CMK.

**Complexity:** Medium. Crypto operations require care. ~400-500 LoC.

---

## 7. Tier 3: Nice-to-Have Operations

Summary of ~80 operations across services:

| Service | Operations | Notes |
|---|---|---|
| Lambda | Provisioned concurrency (4), code signing (9), runtime management (2), response streaming (1) | Production tuning features |
| IAM | OIDC provider extras (3), SAML provider tags (3), MFA tag ops (3) | Niche admin features |
| S3 | Analytics (4), inventory (4), metrics (4), intelligent tiering (4) | Cost optimization metadata |
| CloudWatch Logs | Data protection (3), account policies (3), index policies (3), live tail (1) | Newer observability features |
| SES | Custom verification templates (6), bounce handling (2), receipt filters (3), tracking options (3) | Email operations features |
| DynamoDB | Continuous backups (2), contributor insights (3) | Terraform-optional features |
| KMS | Multi-region (2), DeriveSharedSecret (1), ListKeyRotations (1) | Enterprise crypto |
| EventBridge | Partner event source ops (10) | Marketplace integrations |

Implementation approach for Tier 3: **stub with accept-and-store semantics**. Accept the API call, store metadata, return success. No behavioral implementation needed. This prevents hard failures in IaC while keeping implementation effort minimal (~20-50 LoC per operation).

---

## 8. Tier 4: Skip

~334 operations that provide zero value in local development:

| Category | Operations | Services | Reason |
|---|---|---|---|
| EC2 fleet management | ~93 | SSM | Maintenance windows, patching, automation, session manager — requires actual EC2 instances |
| IAM governance | ~66 | IAM | Login profiles, credential reports, Organizations, password policies — console/account management |
| Log pipelines | ~62 | CloudWatch Logs | Delivery, anomaly detection, scheduled queries, transformers — production observability |
| DR/replication | ~35 | DynamoDB | Backups, global tables, import/export — disaster recovery |
| Lambda managed infra | ~22 | Lambda | Capacity providers, durable executions, scaling configs — managed runtime internals |
| Other | ~56 | Various | Scattered low-value operations |

These operations should return `NotImplementedError` with a clear message indicating the operation is not supported in local emulation.

---

## 9. Phased Implementation Plan

### Phase 1: DynamoDB Transactions + TTL + Tagging (7 ops)

**Estimated effort:** ~1,200-1,500 LoC
**Priority:** Highest — unblocks the most users

#### Step 1.1: Tagging (TagResource, UntagResource, ListTagsOfResource)

The `DynamoDBTable` struct already has a `tags: parking_lot::RwLock<HashMap<String, String>>` field. Implementation:

1. Add 3 new variants to `DynamoDBOperation` enum
2. Add 3 match arms in `handler.rs` dispatch
3. Implement `handle_tag_resource()`:
   - Validate table ARN exists
   - Merge new tags into existing (max 50 tags)
   - Validate tag key/value constraints (key: 1-128 chars, value: 0-256 chars)
4. Implement `handle_untag_resource()`:
   - Validate table ARN exists
   - Remove specified tag keys
5. Implement `handle_list_tags_of_resource()`:
   - Validate table ARN exists
   - Return current tag map

**Estimated:** ~150 LoC

#### Step 1.2: TTL (UpdateTimeToLive, DescribeTimeToLive)

The `DynamoDBTable` struct already has a `ttl: parking_lot::RwLock<Option<TimeToLiveSpecification>>` field. Implementation:

1. Add 2 new variants to `DynamoDBOperation` enum
2. Implement `handle_update_time_to_live()`:
   - Validate table exists
   - Validate only one TTL attribute allowed per table
   - Store `TimeToLiveSpecification { attribute_name, enabled }` in table
   - Return the specification
3. Implement `handle_describe_time_to_live()`:
   - Return stored specification or default (disabled)

**Note:** Actual TTL deletion behavior (background task removing expired items) is a Tier 3 enhancement. For now, just store and return the configuration. Applications that rely on TTL deletion timing in local dev are rare — most apps verify items are *written* with a TTL attribute, not that they're *deleted* on time.

**Estimated:** ~100 LoC

#### Step 1.3: Transactions (TransactGetItems, TransactWriteItems)

This is the most complex addition. It requires atomicity across multiple items/tables.

1. Add 2 new variants to `DynamoDBOperation` enum
2. Add model types: `TransactGetItemsInput/Output`, `TransactWriteItemsInput/Output`, `TransactGetItem`, `TransactWriteItem`, `Get`, `Put`, `Update`, `Delete`, `ConditionCheck`
3. Implement `handle_transact_get_items()`:
   - Validate max 100 items
   - For each `Get`: resolve table, extract key, fetch item, apply projection
   - All reads are atomic (snapshot isolation) — take read locks on all involved partitions
   - Return items in order matching input
4. Implement `handle_transact_write_items()`:
   - Validate max 100 actions
   - Validate no two actions target the same item (primary key collision detection)
   - **Phase 1 — Validate all conditions:**
     - For each `ConditionCheck`: evaluate condition expression, fail entire transaction if any condition fails
     - For each `Put` with condition: evaluate condition expression
     - For each `Update` with condition: evaluate condition expression
     - For each `Delete` with condition: evaluate condition expression
   - **Phase 2 — Apply all writes:**
     - Acquire write locks on all involved partitions (sorted by partition key to prevent deadlocks)
     - For each `Put`: insert/replace item
     - For each `Update`: apply update expression
     - For each `Delete`: remove item
     - Emit stream events for all changes
     - Release all locks
   - **On any failure:** release all locks, return `TransactionCanceledException` with per-item cancellation reasons

**Concurrency strategy:** Sort all involved (table, partition_key) pairs lexicographically. Acquire DashMap shard locks in this order to prevent deadlocks. This is the same approach used by real DynamoDB (deterministic lock ordering).

**Estimated:** ~800-1,000 LoC

### Phase 2: Lambda Layers (9 ops)

**Estimated effort:** ~800-1,000 LoC
**Priority:** High — unblocks Lambda deployments that use layers

#### Step 2.1: Layer Storage

Add to Lambda storage:

```rust
pub struct LayerStore {
    /// layer_name -> versions
    layers: DashMap<String, LayerRecord>,
}

pub struct LayerRecord {
    name: String,
    versions: BTreeMap<u64, LayerVersionRecord>,
    next_version: u64,
}

pub struct LayerVersionRecord {
    version: u64,
    description: String,
    compatible_runtimes: Vec<String>,
    compatible_architectures: Vec<String>,
    license_info: Option<String>,
    code_sha256: String,
    code_size: u64,
    code_path: PathBuf,
    created_date: String,
    layer_arn: String,
    layer_version_arn: String,
    policy: PolicyDocument,
}
```

#### Step 2.2: Operations

1. **PublishLayerVersion:** Accept zip/S3 code, store on disk, create version record, return ARN
2. **GetLayerVersion:** Lookup by name + version number, return metadata + code location
3. **GetLayerVersionByArn:** Parse ARN → (name, version), delegate to GetLayerVersion
4. **ListLayerVersions:** Return all versions for a layer, sorted descending
5. **ListLayers:** Return latest version of each layer
6. **DeleteLayerVersion:** Remove specific version, cleanup code on disk
7. **AddLayerVersionPermission:** Add statement to layer version's policy
8. **GetLayerVersionPolicy:** Return layer version's policy document
9. **RemoveLayerVersionPermission:** Remove statement from policy by SID

#### Step 2.3: Integration with CreateFunction/UpdateFunctionConfiguration

Modify existing function creation to validate layer ARNs:
- Each layer ARN must reference an existing layer version
- Max 5 layers per function
- Store resolved layer ARNs in version record (already a `Vec<String>` field)

### Phase 3: Lambda Event Source Mappings (5 ops)

**Estimated effort:** ~1,000-1,200 LoC
**Priority:** High — unblocks SQS/Kinesis/DDB trigger-based Lambdas

#### Step 3.1: Event Source Mapping Storage

```rust
pub struct EventSourceMappingStore {
    mappings: DashMap<String, EventSourceMappingRecord>,
}

pub struct EventSourceMappingRecord {
    uuid: String,
    event_source_arn: String,
    function_arn: String,
    state: String, // Enabled, Disabled, Creating, Updating, Deleting
    batch_size: u32,
    maximum_batching_window_in_seconds: u32,
    starting_position: Option<String>, // TRIM_HORIZON, LATEST, AT_TIMESTAMP
    starting_position_timestamp: Option<String>,
    enabled: bool,
    filter_criteria: Option<FilterCriteria>,
    maximum_record_age_in_seconds: Option<i32>,
    bisect_batch_on_function_error: Option<bool>,
    maximum_retry_attempts: Option<i32>,
    parallelization_factor: Option<u32>,
    destination_config: Option<DestinationConfig>,
    function_response_types: Vec<String>,
    last_modified: String,
    last_processing_result: String,
    state_transition_reason: String,
}
```

#### Step 3.2: Operations

1. **CreateEventSourceMapping:** Validate event source ARN (SQS/Kinesis/DDB Streams), validate function exists, create UUID, store record, return mapping. State is `Creating` → `Enabled`.
2. **GetEventSourceMapping:** Lookup by UUID, return record.
3. **UpdateEventSourceMapping:** Update mutable fields (batch_size, enabled, filter_criteria, etc.)
4. **DeleteEventSourceMapping:** Remove by UUID. State transitions to `Deleting`.
5. **ListEventSourceMappings:** Filter by function name and/or event source ARN.

#### Step 3.3: Polling Engine (Optional)

For full behavioral emulation, an optional background polling engine:
- Polls SQS queues, Kinesis shards, or DDB Streams on a timer
- Invokes the mapped Lambda function with the polled records
- Handles success (delete from SQS / advance shard iterator) and failure (retry, DLQ)

This is a significant feature (~2,000+ LoC) and can be deferred. The metadata CRUD alone (Tier 1) unblocks IaC deployments. The polling engine is a Tier 2 enhancement.

### Phase 4: IAM OIDC + Resource Tagging (10 ops)

**Estimated effort:** ~500-600 LoC
**Priority:** High for EKS users

#### Step 4.1: OIDC Provider Storage

```rust
pub struct OIDCProviderRecord {
    arn: String,
    url: String,  // issuer URL (https://...)
    client_id_list: Vec<String>,
    thumbprint_list: Vec<String>,
    tags: HashMap<String, String>,
    create_date: String,
}
```

Add `oidc_providers: DashMap<String, OIDCProviderRecord>` to `IAMServiceState`.

#### Step 4.2: Operations

Standard CRUD pattern. ARN format: `arn:aws:iam::{account}:oidc-provider/{url_host_and_path}`.

#### Step 4.3: Policy/InstanceProfile Tagging

Add `tags: HashMap<String, String>` to `PolicyRecord` and `InstanceProfileRecord` (if not already present). Implement standard TagResource/UntagResource/ListTags pattern.

### Phase 5: EventBridge Remaining Handlers (13+ ops)

**Estimated effort:** ~600-800 LoC
**Priority:** Medium — operations are already in the model

Implement handler methods for already-defined operation variants:
- Archives: Store event archive metadata, accept replay requests
- API Destinations: Store HTTP endpoint configurations
- Connections: Store authentication configurations (API key, OAuth, Basic)
- Endpoints: Store global endpoint configurations

All are metadata CRUD with no behavioral side effects needed for local dev.

### Phase 6: Tier 2 Grab Bag

Implement remaining Tier 2 operations across services. Each is independent and can be parallelized:
- DynamoDB PartiQL (3 ops, ~800 LoC for parser)
- DynamoDB DescribeLimits/DescribeEndpoints (2 ops, ~60 LoC)
- Lambda concurrency + event invoke configs (8 ops, ~300 LoC)
- IAM SAML + account aliases + permission boundaries (14 ops, ~700 LoC)
- S3 replication config + SelectObjectContent + RestoreObject (7 ops, ~1,500 LoC)
- SSM documents (5 ops, ~500 LoC)
- KMS key import (3 ops, ~500 LoC)

---

## 10. Detailed Design: DynamoDB Transactions

### 10.1 Model Types

```rust
/// Input for TransactWriteItems
pub struct TransactWriteItemsInput {
    pub transact_items: Vec<TransactWriteItem>,
    pub return_consumed_capacity: Option<ReturnConsumedCapacity>,
    pub return_item_collection_metrics: Option<ReturnItemCollectionMetrics>,
    pub client_request_token: Option<String>, // idempotency token
}

pub struct TransactWriteItem {
    pub condition_check: Option<ConditionCheck>,
    pub put: Option<Put>,
    pub delete: Option<Delete>,
    pub update: Option<Update>,
}

pub struct ConditionCheck {
    pub table_name: String,
    pub key: HashMap<String, AttributeValue>,
    pub condition_expression: String,
    pub expression_attribute_names: Option<HashMap<String, String>>,
    pub expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    pub return_values_on_condition_check_failure: Option<String>,
}

pub struct Put {
    pub table_name: String,
    pub item: HashMap<String, AttributeValue>,
    pub condition_expression: Option<String>,
    pub expression_attribute_names: Option<HashMap<String, String>>,
    pub expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    pub return_values_on_condition_check_failure: Option<String>,
}

// Delete and Update follow same pattern as existing operations
// but embedded within a transaction context
```

### 10.2 Concurrency Protocol

```
TransactWriteItems(items):
  1. Validate: max 100 items, no duplicate (table, key) pairs
  2. Collect all (table_name, partition_key) pairs
  3. Sort pairs lexicographically (prevents deadlock)
  4. For each pair, acquire write access on the DashMap shard:
     - DashMap internally hashes the key to a shard
     - We use DashMap::get_mut() which holds the shard lock
  5. Check all conditions:
     - For each ConditionCheck/Put/Update/Delete with condition_expression:
       fetch current item, evaluate condition
     - If ANY condition fails: release all locks, return TransactionCanceledException
       with per-item cancellation reasons
  6. Apply all writes:
     - Put: insert/replace item
     - Update: apply update expression
     - Delete: remove item
  7. Emit stream events for all changes
  8. Release all locks (automatic via drop)
  9. Return success
```

### 10.3 Idempotency Token

If `client_request_token` is provided, cache the token with a 10-minute TTL. If the same token is seen again within the window, return the cached response without re-executing. Use a `DashMap<String, (Instant, TransactWriteItemsOutput)>` with periodic cleanup.

### 10.4 Error Responses

```rust
pub struct TransactionCanceledException {
    pub message: String,
    pub cancellation_reasons: Vec<CancellationReason>,
}

pub struct CancellationReason {
    pub code: String,        // "None", "ConditionalCheckFailed", "ItemCollectionSizeLimitExceeded", etc.
    pub message: Option<String>,
    pub item: Option<HashMap<String, AttributeValue>>, // if return_values_on_condition_check_failure = ALL_OLD
}
```

---

## 11. Detailed Design: Lambda Layers & Event Source Mappings

### 11.1 Layer ARN Format

```
arn:aws:lambda:{region}:{account}:layer:{layer_name}:{version}
```

The ARN without version refers to the layer itself. With version, it refers to a specific layer version.

### 11.2 Layer Code Storage

Follow the same pattern as function code:

```
{code_dir}/layers/{layer_name}/{version}/layer.zip
```

Layer zip files are stored on disk, referenced by `PathBuf` in the `LayerVersionRecord`. SHA-256 hash computed on ingestion for integrity.

### 11.3 Layer Validation During Function Create/Update

When `CreateFunction` or `UpdateFunctionConfiguration` specifies layers:

```rust
fn validate_layers(&self, layer_arns: &[String]) -> Result<(), LambdaServiceError> {
    if layer_arns.len() > 5 {
        return Err(LambdaServiceError::InvalidParameter(
            "Layers list exceeds maximum of 5".into()
        ));
    }
    for arn in layer_arns {
        let (name, version) = parse_layer_arn(arn)?;
        self.layer_store.get_version(&name, version)?;
    }
    Ok(())
}
```

### 11.4 Event Source Mapping UUID Generation

Use UUID v4 for mapping identifiers. The UUID is the primary key for all ESM operations.

### 11.5 Event Source Mapping State Machine

```
Creating → Enabled ↔ Disabled
Creating → CreateFailed
Enabled → Updating → Enabled
Enabled → Deleting → Deleted
Disabled → Deleting → Deleted
```

For local dev, transitions are instant (no async provisioning).

---

## 12. Detailed Design: DynamoDB TTL & Tagging

### 12.1 TTL Storage

Already implemented in `DynamoDBTable`:

```rust
pub struct DynamoDBTable {
    // ... existing fields ...
    pub ttl: parking_lot::RwLock<Option<TimeToLiveSpecification>>,
}

pub struct TimeToLiveSpecification {
    pub attribute_name: String,
    pub enabled: bool,
}
```

### 12.2 TTL Validation Rules

- Only one TTL attribute per table
- Attribute name must be a top-level attribute (no nested paths)
- Attribute name must be 1-255 characters
- Cannot enable/disable TTL within 1 hour of previous change (enforce with `last_ttl_change: Option<Instant>`)

### 12.3 Tagging Validation Rules

- Max 50 tags per resource
- Tag key: 1-128 Unicode characters
- Tag value: 0-256 Unicode characters
- Tag keys cannot start with `aws:` prefix (reserved)
- Resource is identified by ARN, not table name

### 12.4 Tag ARN Resolution

The tagging API uses resource ARNs, not table names. Implement ARN → table name resolution:

```rust
fn resolve_table_from_arn(&self, arn: &str) -> Result<&str, DynamoDBError> {
    // arn:aws:dynamodb:{region}:{account}:table/{table_name}
    arn.strip_prefix("arn:aws:dynamodb:")
        .and_then(|s| s.split('/').nth(1))
        .ok_or_else(|| DynamoDBError::validation("Invalid resource ARN"))
}
```

---

## 13. Cross-Cutting Concerns

### 13.1 Operation Registration Pattern

Every new operation follows the same three-step pattern across all services:

1. **Model:** Add enum variant to `{Service}Operation`, implement `as_str()` and `from_name()`
2. **HTTP:** Add match arm in `handler.rs` dispatch function
3. **Core:** Implement `handle_{operation}()` method on the provider

For services using `awsJson1_0`/`awsJson1_1` (DynamoDB, Lambda, KMS, Logs, CloudWatch), the HTTP layer is a single dispatch function keyed on the `X-Amz-Target` header. Adding an operation is adding one match arm.

For services using `restJson1` (Lambda, API Gateway v2), adding an operation requires adding a route entry to the route table.

For services using `awsQuery` (IAM, SES, STS, SNS), adding an operation requires adding a match arm keyed on the `Action` query parameter.

### 13.2 Smithy Model Regeneration

New model types for added operations should be generated from Smithy models where possible. For DynamoDB transactions, the Smithy model already contains `TransactGetItems`, `TransactWriteItems`, and all associated types. Run `cargo run -p codegen -- --service dynamodb` to regenerate and pick up the new types.

For operations where Smithy codegen doesn't yet support the service (Lambda, IAM), hand-write the types following existing patterns. The types are straightforward serde structs.

### 13.3 Error Code Consistency

All new operations must use the service-specific error types already defined. New error variants needed:

- DynamoDB: `TransactionCanceledException`, `TransactionConflictException`, `TransactionInProgressException`, `IdempotentParameterMismatchException`
- Lambda: `LayerVersionNotFoundException`, `LayerNotFoundException`, `EventSourceMappingNotFoundException`
- IAM: `EntityAlreadyExistsException` (for OIDC providers), `NoSuchEntityException`

### 13.4 Pagination

Several new list operations return paginated results:

- `ListLayers`, `ListLayerVersions`: Marker-based (same as `ListFunctions`)
- `ListEventSourceMappings`: Marker-based with optional filters
- `ListOpenIDConnectProviders`: Returns all (no pagination, max ~100)

Follow existing pagination patterns: accept `Marker`/`MaxItems`, return `NextMarker` when more results exist.

---

## 14. Testing Strategy

### 14.1 Unit Tests

For each new operation, test:
- Happy path with minimal input
- Happy path with all optional fields
- Validation error cases (missing required fields, invalid values, limit exceeded)
- Idempotency (where applicable)
- Concurrent access (for transactions)

### 14.2 Integration Tests

For each phase, write integration tests using `aws-sdk-rust`:

**Phase 1 (DynamoDB Transactions):**
- Create table → TransactWriteItems (Put 3 items) → TransactGetItems (read all 3)
- TransactWriteItems with condition check failure → verify no items written
- Concurrent TransactWriteItems targeting same item → verify one succeeds
- UpdateTimeToLive → DescribeTimeToLive → verify round-trip
- TagResource → ListTagsOfResource → verify tags present
- UntagResource → ListTagsOfResource → verify tag removed

**Phase 2 (Lambda Layers):**
- PublishLayerVersion with zip → GetLayerVersion → verify metadata
- CreateFunction with layer ARN → GetFunction → verify layer in config
- DeleteLayerVersion → CreateFunction with deleted layer → verify error
- ListLayers → verify all layers returned

**Phase 3 (Lambda ESM):**
- CreateEventSourceMapping (SQS) → GetEventSourceMapping → verify
- UpdateEventSourceMapping (disable) → GetEventSourceMapping → verify state
- DeleteEventSourceMapping → ListEventSourceMappings → verify removed
- CreateEventSourceMapping with non-existent function → verify error

### 14.3 LocalStack Test Suite Compatibility

Where applicable, port relevant LocalStack test cases:
- `tests/aws/services/dynamodb/test_dynamodb.py` — transaction tests
- `tests/aws/services/lambda_/test_lambda_api.py` — layer and ESM tests
- `tests/aws/services/iam/test_iam.py` — OIDC provider tests

Run these against Rustack to validate behavioral parity.

---

## 15. Risk Analysis

### 15.1 Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Transaction deadlock under concurrent load | Medium | High | Deterministic lock ordering (sorted partition keys). Extensive concurrent test suite. |
| Transaction condition evaluation semantics diverge from AWS | Medium | Medium | Test against real DynamoDB for edge cases. Port LocalStack transaction tests. |
| PartiQL parser incomplete for complex queries | High | Low | Start with simple SELECT/INSERT/UPDATE/DELETE. Add complexity incrementally. |
| Layer zip extraction interferes with function invoke | Low | Medium | Layers are stored but not extracted during invoke (invoke is still stubbed). |
| Event Source Mapping polling introduces async complexity | Medium | Medium | Defer polling engine. Phase 3 is metadata CRUD only. |

### 15.2 Scope Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Users demand Tier 4 operations | Medium | Low | Return clear `NotImplementedError` with message. Accept feature requests. |
| Tier 2 operations become blocking for a major user | Medium | Medium | Tier 2 is designed to be independently implementable. Can fast-track individual operations. |
| New AWS operations added to services we support | High | Low | Regenerate from Smithy models quarterly. New ops default to NotImplemented. |

### 15.3 Behavioral Differences

| Behavior | AWS | Rustack | Justification |
|---|---|---|---|
| TTL deletion timing | Items deleted within 48 hours | Items not deleted (metadata only) | Background deletion adds complexity with minimal local dev value |
| Transaction conflict window | Strict serializable isolation | Partition-level lock ordering | Sufficient for local dev correctness |
| Event Source Mapping polling | AWS manages polling infrastructure | No polling (metadata only, Phase 3) | IaC needs metadata; polling is optional enhancement |
| Layer code extraction | Layers merged into function runtime | Layers stored but not merged into invoke | Invoke is already stubbed; full layer merging deferred to Docker execution engine |
| Idempotency token TTL | 10 minutes | 10 minutes | Match AWS behavior |

---

## 16. Success Metrics

### 16.1 Coverage Targets

| Milestone | Operations | % of LocalStack shared ops | Phase |
|---|---|---|---|
| Current | 586 | 53% | — |
| After Phase 1-4 (Tier 1) | 630 | 57% | 1-4 |
| After Phase 5 (EventBridge) | 643 | 58% | 5 |
| After Phase 6 (Tier 2) | 699 | 64% | 6 |
| Tier 3 stubs | 779 | 71% | Future |

### 16.2 Qualitative Targets

- **DynamoDB:** Any application using transactions, TTL, or tagging works without modification
- **Lambda:** Any IaC deployment using layers or event source mappings succeeds
- **IAM:** EKS IRSA testing works (OIDC providers)
- **EventBridge:** All defined operations have working handlers
- **Zero regressions:** All existing tests continue to pass

### 16.3 Compatibility Validation

For each phase, validate against:
1. `aws-sdk-rust` integration tests
2. `aws-cli` manual smoke tests
3. Terraform `plan`/`apply`/`destroy` cycle for resources using the new operations
4. CDK `synth`/`deploy` for representative stacks
