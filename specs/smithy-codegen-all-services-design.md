# Rustack Universal Smithy Codegen: Design Spec

**Date:** 2026-03-18
**Status:** Draft / RFC
**Depends on:** [smithy-s3-redesign-design.md](./smithy-s3-redesign-design.md)
**Scope:** Extend the existing Smithy-to-Rust code generator from S3-only to all Rustack services (DynamoDB, SQS, SSM, SNS, Lambda, EventBridge, and future services). Produce a single configurable tool that reads AWS Smithy JSON AST models and generates model crates with protocol-aware serde attributes.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Motivation](#2-motivation)
3. [Goals and Non-Goals](#3-goals-and-non-goals)
4. [Current State Analysis](#4-current-state-analysis)
5. [Architecture Overview](#5-architecture-overview)
6. [Service Configuration Model](#6-service-configuration-model)
7. [Protocol-Aware Code Generation](#7-protocol-aware-code-generation)
8. [Shape Resolution Changes](#8-shape-resolution-changes)
9. [Output File Generation Changes](#9-output-file-generation-changes)
10. [Error Type Generation](#10-error-type-generation)
11. [Smithy Model Acquisition](#11-smithy-model-acquisition)
12. [Migration Strategy](#12-migration-strategy)
13. [Makefile Integration](#13-makefile-integration)
14. [Testing Strategy](#14-testing-strategy)
15. [Phased Implementation Plan](#15-phased-implementation-plan)
16. [Risk Analysis](#16-risk-analysis)

---

## 1. Executive Summary

The Rustack codegen tool currently generates Rust types from the AWS S3 Smithy JSON AST. All other services (DynamoDB, SQS, SSM, SNS, Lambda, EventBridge) use hand-written model crates. This creates maintenance burden, risks drift from AWS API changes, and produces inconsistent patterns across services.

This spec proposes extending the codegen tool to be **service-agnostic**: a single binary that reads any AWS Smithy model and a service configuration file, producing a complete `rustack-{service}-model` crate. The key challenge is handling **four distinct AWS protocols** with different serialization requirements.

**Key design decisions:**
- **TOML-based service configuration** -- each service declares its operations, protocol, namespace, error codes, and customizations in a `.toml` file, replacing the hardcoded S3 operation list
- **Protocol-aware serde attributes** -- the codegen detects the service protocol and emits the correct serde derive macros and rename strategies
- **Preserve existing hand-written customizations** -- services with custom logic (DynamoDB's `AttributeValue`, SQS's `base64_option`) use `#[codegen(skip)]` markers or overlay files that survive regeneration
- **Incremental adoption** -- each service migrates independently; hand-written and generated models can coexist during transition

---

## 2. Motivation

### 2.1 Current Problems

| Problem | Impact | Example |
|---------|--------|---------|
| **7 hand-written model crates** | ~4,500 lines of boilerplate per service | SSM model has 287 lines of input.rs, 195 lines of output.rs -- all repetitive serde structs |
| **Inconsistent patterns** | Each service uses slightly different conventions | SSM uses `SsmErrorCode` enum, DynamoDB uses fully-qualified error types, SQS has `is_sender_fault()` |
| **No serde derives on S3 structs** | S3 input/output structs lack `Serialize`/`Deserialize` | Generated S3 types use HTTP bindings instead of serde; all other services use serde |
| **Drift from AWS APIs** | New operations require manual addition | When AWS adds a new EventBridge operation, someone must hand-write the input/output types |
| **Error-prone field naming** | PascalCase serde renames must match exactly | A typo in `#[serde(rename = "ResourceARN")]` causes silent deserialization failures |
| **No type validation against AWS** | No way to verify hand-written types match the official model | A field typed as `Option<String>` may actually be required per the Smithy model |

### 2.2 Value Proposition

| Benefit | Quantified Impact |
|---------|-------------------|
| **Eliminate ~20,000 lines of hand-written code** | 6 services × ~3,500 lines average = ~21,000 lines replaced |
| **Single source of truth** | All types derived from official AWS Smithy models |
| **Consistent error handling** | Unified error type pattern across all services |
| **Automated AWS API updates** | `make codegen-update` downloads latest models and regenerates |
| **New service in minutes** | Adding a new service = download Smithy model + write 30-line TOML config |

### 2.3 What We Keep

The codegen does NOT replace:
- **HTTP layer** (`-http` crates): Routing, dispatching, and protocol handling remain hand-written
- **Core business logic** (`-core` crates): Providers, storage, pattern matching, etc.
- **Handler dispatch** (`handler.rs`): The match statement wiring operations to provider methods
- **Server integration** (`main.rs`, `service.rs`): Gateway routing and service wiring

Only the **model crate** (`-model`) is generated. This is the lowest-risk, highest-value target.

---

## 3. Goals and Non-Goals

### 3.1 Goals

1. **One codegen binary for all services** -- no per-service codegen tools
2. **TOML-based service configuration** -- declarative service definition
3. **Protocol-aware serde generation** -- correct attributes for each AWS protocol
4. **Operation filtering** -- generate only the operations each service implements
5. **Error type generation from Smithy model** -- extract error shapes and HTTP status codes
6. **Stable output** -- deterministic, sorted, formatted output for clean diffs
7. **Overlay files** -- allow hand-written extensions that survive regeneration
8. **Makefile targets** -- `make codegen`, `make codegen-{service}`, `make codegen-update`

### 3.2 Non-Goals

1. **HTTP layer generation** -- router, service, dispatch remain hand-written
2. **Business logic generation** -- provider implementations are not generated
3. **Runtime protocol handling** -- serde does the work; no runtime protocol library needed
4. **Custom DynamoDB type system** -- `AttributeValue` is too complex for generic codegen; use overlay
5. **100% Smithy fidelity** -- we generate what we need, not every shape in the model

---

## 4. Current State Analysis

### 4.1 Codegen Architecture (S3-Only)

```
codegen/
├── Cargo.toml
├── smithy-model/
│   └── s3.json          # 44K lines, AWS S3 Smithy JSON AST
└── src/
    ├── main.rs           # Entry point: read model → resolve → generate → write
    ├── model.rs          # Smithy JSON AST types (Shape, Operation, Structure, etc.)
    ├── shapes.rs         # Shape resolution: Smithy → Rust type mapping
    └── codegen.rs        # Code generation: Rust source text emission
```

**Current limitations:**
- `S3_NAMESPACE` hardcoded to `com.amazonaws.s3#`
- `TARGET_OPERATIONS` is a hardcoded list of 70 S3 operation names
- `categorize_operations()` returns S3-specific file categories (bucket, object, multipart, list, config)
- `generate_operations()` emits `S3Operation` enum
- `generate_error()` emits `S3ErrorCode` with hardcoded S3 error codes
- `generate_request()` emits S3-specific `S3Request<T>`, `StreamingBlob`, `Credentials`
- No serde `Serialize`/`Deserialize` derives on input/output structs
- HTTP binding comments but no serde rename attributes

### 4.2 Hand-Written Model Crate Pattern

All 6 hand-written model crates follow this structure:

```
crates/rustack-{service}-model/src/
├── lib.rs         # Module re-exports
├── error.rs       # {Service}ErrorCode enum + {Service}Error struct + macro
├── operations.rs  # {Service}Operation enum with as_str(), from_name()
├── types.rs       # Shared types (Tag, etc.)
├── input.rs       # All input structs (PascalCase serde)
└── output.rs      # All output structs (PascalCase serde)
```

### 4.3 Protocol Matrix

| Service | Protocol | Serde Rename | Dispatch | Timestamp | Error Format |
|---------|----------|-------------|----------|-----------|-------------|
| **S3** | `restXml` | None (HTTP bindings) | HTTP method + path | ISO 8601 | XML `<Error><Code>` |
| **DynamoDB** | `awsJson1.0` | `PascalCase` | `X-Amz-Target: DynamoDB_20120810.{Op}` | Epoch seconds | `{"__type": "com.amazonaws...#Code"}` |
| **SQS** | `awsJson1.0` (awsQueryCompat) | `PascalCase` | `X-Amz-Target: AmazonSQS.{Op}` | ISO 8601 | `{"__type": "Code", "x-amzn-query-error": "Code;Sender"}` |
| **SSM** | `awsJson1.1` | `PascalCase` | `X-Amz-Target: AmazonSSM.{Op}` | Epoch seconds (f64) | `{"__type": "Code", "message": "..."}` |
| **SNS** | `awsQuery` | `PascalCase` | `Action=OpName` in form body | ISO 8601 | XML `<ErrorResponse>` |
| **Lambda** | `restJson1` | `camelCase` | HTTP method + path | ISO 8601 | `{"__type": "Code", "message": "..."}` |
| **EventBridge** | `awsJson1.1` | `PascalCase` | `X-Amz-Target: AWSEvents.{Op}` | ISO 8601 | `{"__type": "Code", "message": "..."}` |

### 4.4 Struct Generation Pattern Comparison

**Generated S3 (no serde):**
```rust
#[derive(Debug, Clone, Default)]
pub struct PutObjectInput {
    pub bucket: String,
    pub key: String,
    pub body: Option<StreamingBlob>,
    pub content_type: Option<String>,
}
```

**Hand-written SSM (with serde):**
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PutParameterInput {
    pub name: String,
    pub value: String,
    #[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
    pub parameter_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
```

**Hand-written Lambda (camelCase serde):**
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFunctionInput {
    pub function_name: String,
    pub runtime: Option<String>,
    pub handler: Option<String>,
    pub code: FunctionCode,
}
```

The codegen must emit the correct serde attributes per protocol.

---

## 5. Architecture Overview

### 5.1 New Pipeline

```
                    ┌──────────────────┐
                    │  Service Config   │  TOML file per service
                    │  (events.toml)    │  declares operations, protocol,
                    └────────┬─────────┘  namespace, customizations
                             │
                    ┌────────▼─────────┐
                    │  Smithy JSON AST  │  Official AWS model
                    │  (events.json)    │  from aws/api-models-aws
                    └────────┬─────────┘
                             │
                    ┌────────▼─────────┐
                    │  Shape Resolver   │  Generic: works with any namespace
                    │  (shapes.rs)      │  Protocol-aware type mapping
                    └────────┬─────────┘
                             │
                    ┌────────▼─────────┐
                    │  Code Generator   │  Protocol-aware serde attributes
                    │  (codegen.rs)     │  Configurable error/operation names
                    └────────┬─────────┘
                             │
              ┌──────────────▼──────────────┐
              │    Generated Model Crate     │
              │  crates/rustack-{svc}-model│
              │  ├── lib.rs                  │
              │  ├── types.rs (serde derives)│
              │  ├── operations.rs           │
              │  ├── error.rs                │
              │  ├── input.rs (serde derives)│
              │  └── output.rs (serde derives│
              └──────────────────────────────┘
```

### 5.2 Crate Layout (Codegen Tool)

```
codegen/
├── Cargo.toml
├── smithy-model/
│   ├── s3.json
│   ├── dynamodb.json
│   ├── sqs.json
│   ├── ssm.json
│   ├── sns.json
│   ├── lambda.json
│   ├── events.json
│   └── (future: kms.json, kinesis.json, logs.json, secretsmanager.json)
├── services/
│   ├── s3.toml
│   ├── dynamodb.toml
│   ├── sqs.toml
│   ├── ssm.toml
│   ├── sns.toml
│   ├── lambda.toml
│   └── events.toml
└── src/
    ├── main.rs         # CLI: reads config + model, runs pipeline
    ├── config.rs       # Service configuration TOML parsing
    ├── model.rs        # Smithy JSON AST types (unchanged)
    ├── shapes.rs       # Generic shape resolution (refactored)
    └── codegen.rs      # Protocol-aware code generation (refactored)
```

---

## 6. Service Configuration Model

### 6.1 TOML Schema

Each service is described by a TOML file:

```toml
# codegen/services/events.toml

[service]
name = "events"                         # Crate suffix: rustack-events-model
display_name = "EventBridge"            # Human-readable name for doc comments
rust_prefix = "Events"                  # Rust type prefix: EventsOperation, EventsError
namespace = "com.amazonaws.cloudwatchevents"  # Smithy namespace (before #)
protocol = "awsJson1_1"                # One of: restXml, awsJson1_0, awsJson1_1, awsQuery, restJson1

[protocol]
serde_rename = "PascalCase"            # rename_all strategy for structs
target_prefix = "AWSEvents"            # X-Amz-Target prefix (for awsJson protocols)
content_type = "application/x-amz-json-1.1"
error_type_field = "__type"            # Field name in error JSON
error_type_format = "short"            # "short" (SSM/Events) or "qualified" (DynamoDB)

[operations]
# Operations to generate, grouped by phase for the operation enum.
# Each phase gets an is_phase{N}() method on the operation enum.
phase0 = [
    "CreateEventBus", "DeleteEventBus", "DescribeEventBus", "ListEventBuses",
    "PutRule", "DeleteRule", "DescribeRule", "ListRules", "EnableRule", "DisableRule",
    "PutTargets", "RemoveTargets", "ListTargetsByRule",
    "PutEvents", "TestEventPattern",
]
phase1 = [
    "TagResource", "UntagResource", "ListTagsForResource",
    "PutPermission", "RemovePermission", "ListRuleNamesByTarget",
]
phase2 = ["UpdateEventBus"]
phase3 = [
    "CreateArchive", "DeleteArchive", "DescribeArchive", "ListArchives", "UpdateArchive",
    "StartReplay", "CancelReplay", "DescribeReplay", "ListReplays",
    # ... (stubs)
]

[errors]
# Error codes not derivable from Smithy model (service-specific additions).
# Format: code = { status = 400, message = "..." }
# Smithy model errors are extracted automatically; these are overrides/additions.
[errors.custom]
MissingAction = { status = 400, message = "Missing required header: X-Amz-Target" }
InvalidAction = { status = 400, message = "Operation is not supported" }

[output]
# Output customizations
dir = "../crates/rustack-events-model/src"  # Where to write generated files

# Fields that should always be serialized (never skip_serializing_if)
always_serialize_arrays = true           # Never skip empty Vec fields in outputs

[input]
# Input customizations
# Fields with special serde rename that differ from PascalCase
[input.field_renames]
# field_path = "SerdeRenameTo"
# e.g., "TagResourceInput.resource_arn" = "ResourceARN"

[overlay]
# Files that are hand-written and should NOT be overwritten by codegen.
# These are merged with generated code at the module level.
preserve = []
# e.g., preserve = ["types_custom.rs"]  # for DynamoDB's AttributeValue
```

### 6.2 Protocol-Specific Configs

**S3 (restXml):**
```toml
[service]
name = "s3"
protocol = "restXml"

[protocol]
serde_rename = "none"                   # No serde on S3 types (HTTP bindings)
emit_http_bindings = true               # Generate HTTP binding comments/annotations
emit_serde_derives = false              # S3 types don't use serde
emit_request_wrapper = true             # Generate S3Request<T>, StreamingBlob
```

**DynamoDB (awsJson1.0):**
```toml
[service]
name = "dynamodb"
protocol = "awsJson1_0"

[protocol]
serde_rename = "PascalCase"
target_prefix = "DynamoDB_20120810"
error_type_format = "qualified"         # "com.amazonaws.dynamodb.v20120810#ValidationException"

[overlay]
preserve = ["types_custom.rs"]          # AttributeValue custom implementation
```

**Lambda (restJson1):**
```toml
[service]
name = "lambda"
protocol = "restJson1"

[protocol]
serde_rename = "camelCase"              # Lambda uses camelCase JSON
emit_route_table = true                 # Generate LambdaRoute table for URL routing
```

**SQS (awsJson1.0 + awsQueryCompat):**
```toml
[service]
name = "sqs"
protocol = "awsJson1_0"

[protocol]
serde_rename = "PascalCase"
target_prefix = "AmazonSQS"
aws_query_compatible = true             # Emit x-amzn-query-error header support

[overlay]
preserve = ["types_custom.rs"]          # base64_option serde module
```

---

## 7. Protocol-Aware Code Generation

### 7.1 Serde Derive Strategy

| Protocol | Shared Types | Input Structs | Output Structs |
|----------|-------------|---------------|----------------|
| `restXml` | `#[derive(Debug, Clone, Default)]` | No serde | No serde |
| `awsJson1_0` | `Serialize, Deserialize` + `rename_all = "PascalCase"` | Same | Same |
| `awsJson1_1` | `Serialize, Deserialize` + `rename_all = "PascalCase"` | Same | Same |
| `awsQuery` | `Serialize, Deserialize` + `rename_all = "PascalCase"` | Same | Same |
| `restJson1` | `Serialize, Deserialize` + `rename_all = "camelCase"` | Same | Same |

### 7.2 Field Attribute Generation

For serde-enabled protocols, emit per-field attributes:

```rust
// Required field (no smithy.api#required trait): wrap in Option
#[serde(skip_serializing_if = "Option::is_none")]
pub description: Option<String>,

// Collection field: default + always serialize (per config)
#[serde(default)]
pub tags: Vec<Tag>,

// Field with name collision or special rename
#[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
pub parameter_type: Option<String>,

// HashMap field: default
#[serde(default)]
pub attributes: HashMap<String, String>,
```

### 7.3 Rename Collision Handling

Some Smithy field names collide with Rust keywords or conventions:

| Smithy Name | Rust Name | Serde Rename |
|-------------|-----------|-------------|
| `Type` | `parameter_type` | `#[serde(rename = "Type")]` |
| `Return` | `r#return` or `return_value` | `#[serde(rename = "Return")]` |
| `Match` | `r#match` | `#[serde(rename = "Match")]` |

The codegen must emit explicit `#[serde(rename = "OriginalName")]` when the snake_case name differs from what `rename_all` would produce. Algorithm:

```
for each field:
    rust_name = to_snake_case(smithy_name)
    expected_by_rename_all = apply_rename_strategy(rust_name, strategy)
    if expected_by_rename_all != smithy_name:
        emit #[serde(rename = "smithy_name")]
```

---

## 8. Shape Resolution Changes

### 8.1 Remove Hardcoded S3 Constants

Replace:
```rust
const S3_NAMESPACE: &str = "com.amazonaws.s3#";
pub const TARGET_OPERATIONS: &[&str] = &[...70 ops...];
```

With config-driven:
```rust
pub struct ServiceConfig {
    pub namespace: String,              // e.g., "com.amazonaws.cloudwatchevents"
    pub operations: Vec<String>,        // Flattened from all phases
    pub protocol: Protocol,
}
```

### 8.2 Generic Shape Resolution

The `resolve_model()` function receives `ServiceConfig` instead of using hardcoded constants:

```rust
pub fn resolve_model(model: &SmithyModel, config: &ServiceConfig) -> Result<ResolvedModel> {
    let namespace_prefix = format!("{}#", config.namespace);
    // ... rest uses namespace_prefix instead of S3_NAMESPACE
}
```

### 8.3 Operation Categories

Replace S3's hardcoded categories (bucket, object, multipart, list, config) with a flat structure. For JSON-protocol services, all operations go in a single `input.rs` and `output.rs` file (no sub-modules needed since JSON services have fewer operations than S3's 70).

Configuration option:
```toml
[output]
# "flat" = single input.rs/output.rs file
# "categorized" = sub-modules like S3
file_layout = "flat"
```

### 8.4 Error Shape Extraction

The Smithy model defines error shapes with HTTP status codes:

```json
{
    "com.amazonaws.cloudwatchevents#ResourceNotFoundException": {
        "type": "structure",
        "members": { "message": { "target": "smithy.api#String" } },
        "traits": {
            "smithy.api#error": "client",
            "smithy.api#httpError": 400
        }
    }
}
```

The codegen extracts:
1. All shapes with `smithy.api#error` trait
2. The HTTP status code from `smithy.api#httpError` trait
3. The error name (short form) from the shape ID
4. The fault type (`client` or `server`) from the `smithy.api#error` value

This replaces the hardcoded error code arrays currently in `codegen.rs`.

---

## 9. Output File Generation Changes

### 9.1 lib.rs Generation

Protocol-dependent module list:

```rust
// restXml (S3): includes request.rs
pub mod error;
pub mod input;
pub mod operations;
pub mod output;
pub mod request;    // S3-only: StreamingBlob, S3Request<T>, Credentials
pub mod types;

// awsJson / restJson: no request.rs
pub mod error;
pub mod input;
pub mod operations;
pub mod output;
pub mod types;
```

Re-exports use configurable prefix:
```rust
// Config: rust_prefix = "Events"
pub use error::{EventsError, EventsErrorCode};
pub use operations::EventsOperation;
```

### 9.2 types.rs Generation

For serde-enabled protocols, emit derives:

```rust
/// EventBridge Tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]  // From config.protocol.serde_rename
pub struct Tag {
    pub key: String,
    pub value: String,
}
```

### 9.3 input.rs / output.rs Generation

For flat layout (JSON services):

```rust
//! Auto-generated from AWS EventBridge Smithy model. DO NOT EDIT.

use serde::{Deserialize, Serialize};
use crate::types::{Tag, Target};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateEventBusInput {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<Tag>,
}
```

### 9.4 operations.rs Generation

```rust
/// All supported EventBridge operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventsOperation {
    // Phase 0
    /// Create an event bus.
    CreateEventBus,
    // ...
}

impl EventsOperation {
    pub fn as_str(&self) -> &'static str { /* ... */ }
    pub fn from_name(name: &str) -> Option<Self> { /* ... */ }
    pub fn is_phase0(&self) -> bool { /* ... */ }
    pub fn is_phase1(&self) -> bool { /* ... */ }
    pub fn is_implemented(&self) -> bool { /* ... */ }
}
```

---

## 10. Error Type Generation

### 10.1 From Smithy Model

Extract error shapes automatically:

```rust
// Generated from Smithy model's error shapes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum EventsErrorCode {
    ResourceNotFoundException,
    ResourceAlreadyExistsException,
    InvalidEventPatternException,
    #[default]
    ValidationException,
    LimitExceededException,
    ConcurrentModificationException,
    InternalException,
    // Custom additions from config:
    MissingAction,
    InvalidAction,
}
```

### 10.2 Status Code Mapping

From `smithy.api#httpError` trait:
- `client` fault + no `httpError` → 400
- `server` fault + no `httpError` → 500
- Explicit `httpError: 404` → 404

### 10.3 Error Struct Pattern

Unified across all services:

```rust
pub struct {Prefix}Error {
    pub code: {Prefix}ErrorCode,
    pub message: String,
    pub status_code: http::StatusCode,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
    // S3-only extensions:
    // pub resource: Option<String>,
    // pub request_id: Option<String>,
}
```

### 10.4 error! Macro

Generated per service:
```rust
#[macro_export]
macro_rules! events_error {
    ($code:ident) => { ... };
    ($code:ident, $msg:expr) => { ... };
}
```

---

## 11. Smithy Model Acquisition

### 11.1 Source

Official AWS Smithy models: `https://github.com/aws/aws-models`

Each service model is a single JSON file in the repository:
- S3: `models/s3/smithy/model.json` (already have as `s3.json`)
- DynamoDB: `models/dynamodb/smithy/model.json`
- SQS: `models/sqs/smithy/model.json`
- SSM: `models/ssm/smithy/model.json`
- SNS: `models/sns/smithy/model.json`
- Lambda: `models/lambda/smithy/model.json`
- EventBridge: `models/eventbridge/smithy/model.json`

### 11.2 Download Script

```makefile
SMITHY_MODELS_REPO = https://raw.githubusercontent.com/aws/aws-models/main

codegen-download:
	@echo "Downloading Smithy models..."
	@curl -sL $(SMITHY_MODELS_REPO)/models/s3/smithy/model.json -o codegen/smithy-model/s3.json
	@curl -sL $(SMITHY_MODELS_REPO)/models/dynamodb/smithy/model.json -o codegen/smithy-model/dynamodb.json
	@curl -sL $(SMITHY_MODELS_REPO)/models/sqs/smithy/model.json -o codegen/smithy-model/sqs.json
	@curl -sL $(SMITHY_MODELS_REPO)/models/ssm/smithy/model.json -o codegen/smithy-model/ssm.json
	@curl -sL $(SMITHY_MODELS_REPO)/models/sns/smithy/model.json -o codegen/smithy-model/sns.json
	@curl -sL $(SMITHY_MODELS_REPO)/models/lambda/smithy/model.json -o codegen/smithy-model/lambda.json
	@curl -sL $(SMITHY_MODELS_REPO)/models/eventbridge/smithy/model.json -o codegen/smithy-model/events.json
	@echo "Done."
```

### 11.3 .gitignore

Smithy model files are large (1-5 MB each). Options:
- **Option A**: Check them in (simple, ~30 MB total, ensures reproducible builds)
- **Option B**: Download on demand (saves repo size, requires network)
- **Recommendation**: Option A -- check them in. They change infrequently and are critical for codegen reproducibility.

---

## 12. Migration Strategy

### 12.1 Approach: Parallel Generation with Diff Validation

For each service migration:

1. **Generate alongside existing**: Run codegen, output to a temporary directory
2. **Diff against hand-written**: Compare generated types with existing hand-written ones
3. **Fix discrepancies**: Adjust config or codegen until output matches semantics
4. **Replace**: Swap generated files in, update Cargo.toml if deps change
5. **Validate**: Run full test suite, including integration tests

### 12.2 Migration Order

| Order | Service | Complexity | Reason |
|-------|---------|-----------|--------|
| 1 | **SSM** | Low | Simplest JSON service, 13 ops, no custom types |
| 2 | **EventBridge** | Low | Standard JSON service, 57 ops, no custom types |
| 3 | **SNS** | Medium | awsQuery protocol adds complexity |
| 4 | **Lambda** | Medium | restJson1 with route table |
| 5 | **SQS** | Medium | base64_option overlay needed |
| 6 | **DynamoDB** | High | AttributeValue custom type, complex expressions |
| 7 | **S3** | Already done | Existing codegen, just add serde for parity |

### 12.3 Overlay System for Custom Types

Some services need hand-written types that are too complex for generic codegen:

**DynamoDB's AttributeValue:**
```
crates/rustack-dynamodb-model/src/
├── types.rs              # GENERATED: standard types (Tag, etc.)
├── types_custom.rs       # HAND-WRITTEN: AttributeValue enum
└── lib.rs                # GENERATED: includes both modules
```

The overlay is declared in the TOML config:
```toml
[overlay]
preserve = ["types_custom.rs"]
extra_modules = ["types_custom"]
```

The generated `lib.rs` includes:
```rust
pub mod types;
pub mod types_custom;  // Hand-written, not overwritten
```

---

## 13. Makefile Integration

```makefile
# Generate model for a single service
codegen-%:
	@cd codegen && cargo run -- --service $* \
		--model smithy-model/$*.json \
		--config services/$*.toml \
		--output ../crates/rustack-$*-model/src
	@cargo +nightly fmt -p rustack-$*-model

# Generate all service models
codegen: codegen-s3 codegen-dynamodb codegen-sqs codegen-ssm codegen-sns \
         codegen-lambda codegen-events

# Download latest Smithy models from AWS
codegen-download:
	@echo "Downloading Smithy models from aws/aws-models..."
	# ... curl commands ...

# Download and regenerate everything
codegen-update: codegen-download codegen
```

---

## 14. Testing Strategy

### 14.1 Codegen Unit Tests

- Parse each service's TOML config
- Verify operation enum generation covers all phases
- Verify serde rename strategy produces correct attributes
- Verify error code extraction from Smithy model
- Verify field optionality rules (required vs optional vs collection)

### 14.2 Snapshot Testing

For each service, maintain a snapshot of expected generated output:

```
codegen/tests/snapshots/
├── events_operations.rs.snap
├── events_error.rs.snap
├── ssm_input.rs.snap
└── ...
```

Run `cargo test` in the codegen crate to compare generated output against snapshots. Update snapshots with `cargo test -- --update`.

### 14.3 Integration Validation

After regeneration, the full test suite (`cargo nextest run --all-features`) must pass. This validates that the generated types are wire-compatible with the hand-written ones.

### 14.4 Roundtrip Serde Tests

For each generated input/output struct, verify that:
1. Deserializing from a known AWS JSON payload produces the expected struct
2. Serializing the struct back produces the original JSON (modulo field ordering)

---

## 15. Phased Implementation Plan

### Phase 1: Refactor Codegen to Be Config-Driven (No New Services)

**Goal**: Make the existing S3 codegen work with a TOML config file instead of hardcoded constants.

- Extract `ServiceConfig` struct and TOML parser (`config.rs`)
- Create `codegen/services/s3.toml` with current S3 settings
- Refactor `shapes.rs` to take `ServiceConfig` instead of constants
- Refactor `codegen.rs` to use config for naming and protocol
- S3 output must be byte-identical to current output
- Update `Makefile` to use `--service s3 --config services/s3.toml`

### Phase 2: Add Serde Generation for JSON Protocols

**Goal**: Generate types with correct serde derives for `awsJson1_0` / `awsJson1_1` protocols.

- Add `Protocol` enum to config: `RestXml`, `AwsJson1_0`, `AwsJson1_1`, `AwsQuery`, `RestJson1`
- Add serde derive emission to `write_struct()`
- Add `skip_serializing_if` for `Option` fields
- Add `rename_all` attribute based on protocol
- Add explicit `#[serde(rename = ...)]` for name collisions
- Generate flat `input.rs` / `output.rs` (no sub-modules) for JSON protocols

### Phase 3: Add Error Extraction from Smithy Model

**Goal**: Extract error types from Smithy model shapes instead of hardcoding.

- Parse `smithy.api#error` and `smithy.api#httpError` traits
- Generate error enum from extracted errors + custom additions
- Generate error struct with unified pattern
- Generate `{service}_error!` macro

### Phase 4: Migrate SSM and EventBridge

**Goal**: First two JSON services generated from Smithy models.

- Download SSM and EventBridge Smithy models
- Create `ssm.toml` and `events.toml` configs
- Generate, diff against hand-written, fix discrepancies
- Replace hand-written model crates
- Run full test suite

### Phase 5: Migrate Remaining Services

**Goal**: Lambda, SQS, SNS, DynamoDB.

- Handle `restJson1` (Lambda) with camelCase rename
- Handle `awsQuery` (SNS) with PascalCase rename
- Handle `awsQueryCompatible` (SQS) with overlay for `base64_option`
- Handle DynamoDB with overlay for `AttributeValue`

### Phase 6: Model Update Automation

**Goal**: `make codegen-update` downloads latest models and regenerates.

- Add `codegen-download` target
- Add CI job that runs codegen and checks for uncommitted changes
- Document the model update process

---

## 16. Risk Analysis

### 16.1 Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Generated types break serde compatibility | High | High | Roundtrip serde tests; diff against hand-written before replacing |
| Smithy model has shapes not handled by resolver | Medium | Medium | Log warnings for unresolved shapes; fallback to `serde_json::Value` |
| Field ordering changes break JSON output | Medium | Low | Use `BTreeMap` for stable ordering; serde doesn't depend on order |
| Custom types (AttributeValue) can't be generated | Known | Medium | Overlay system preserves hand-written files |
| S3 serde addition breaks XML parsing | Low | High | S3 keeps `emit_serde_derives = false`; serde is opt-in |
| Namespace format differs between services | Low | Low | Config-driven namespace; no hardcoded assumptions |

### 16.2 Migration Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Field optionality differs between Smithy and hand-written | High | Medium | Audit each service; Smithy `@required` is authoritative |
| Generated error codes miss custom additions | Medium | Medium | `[errors.custom]` section in TOML config |
| Codegen changes require updating 7 configs | Low | Low | Config files are stable once written |
| Smithy model update introduces new required fields | Low | Medium | CI job detects model changes; review before merging |

---
