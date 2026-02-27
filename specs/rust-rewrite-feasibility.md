# LocalStack Rust Rewrite: Feasibility Analysis & Implementation Spec

**Author:** Claude (AI-assisted analysis)
**Date:** 2026-02-26
**Status:** Draft / RFC
**Scope:** Full codebase analysis of LocalStack with Rust rewrite strategy

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Current Architecture Analysis](#2-current-architecture-analysis)
3. [Feasibility Assessment](#3-feasibility-assessment)
4. [Rust Ecosystem Readiness](#4-rust-ecosystem-readiness)
5. [Architecture Design: RustStack](#5-architecture-design-ruststack)
6. [Migration Strategy](#6-migration-strategy)
7. [Phase 1: Core Framework](#7-phase-1-core-framework)
8. [Phase 2: Priority Services](#8-phase-2-priority-services)
9. [Phase 3: Full Migration](#9-phase-3-full-migration)
10. [Risk Analysis](#10-risk-analysis)
11. [Estimated Effort](#11-estimated-effort)
12. [Recommendation](#12-recommendation)

---

## 1. Executive Summary

**Question:** Can LocalStack be rewritten in Rust? Should it be?

**Answer:** Yes, it is technically feasible, and the Rust ecosystem is surprisingly
well-suited for this task. AWS's own `smithy-rs` project can auto-generate type-safe
server stubs from the same Smithy models that define every AWS service. The
`aws-smithy-http-server-python` crate even provides a hybrid Rust-server/Python-handler
bridge via PyO3 -- enabling an incremental migration rather than a risky big-bang rewrite.

**Expected gains:**
- 30-50x latency reduction for request handling
- ~8x memory reduction (83 MiB -> ~10 MiB for HTTP layer)
- Near-instant cold start (vs. multi-second Python import overhead)
- Single binary distribution (no Python runtime dependency)
- True parallelism (no GIL)

**Recommended approach:** Incremental, service-by-service migration starting with a Rust
core framework and high-value services (SQS, S3, SNS, KMS), using PyO3 to bridge
existing Python service logic during the transition.

---

## 2. Current Architecture Analysis

### 2.1 Codebase Metrics

| Metric                          | Value         |
|---------------------------------|---------------|
| Total Python LOC (main package) | 342,905       |
| Python source files             | 1,202         |
| Test LOC                        | 192,924       |
| Test files                      | 733           |
| AWS services implemented        | 48+           |
| API handler methods             | 265           |
| Runtime dependencies            | 111 packages  |

### 2.2 Service Complexity (LOC)

```
CloudFormation   ████████████████████████████████████████  41,573
StepFunctions    ███████████████████████████████████████   39,454
Lambda           ████████████████                          16,126
API Gateway      ███████████████                           15,835
S3               ████████████                              12,350
Events           ███████                                    7,025
DynamoDB         █████                                      5,840
SNS              █████                                      5,071
SQS              ████                                       4,842
EC2              ████                                       4,243
OpenSearch       ███                                        3,337
KMS              ███                                        3,129
IAM              ███                                        3,014
CloudWatch       ███                                        3,006
SecretsManager   █                                          1,664
SSM              █                                          1,406
Firehose         █                                          1,175
Route53Resolver  █                                          1,099
Kinesis          █                                            983
Logs             █                                            938
```

### 2.3 Architecture Overview

```
                     ┌──────────────────────────────────────┐
                     │     AWS SDK / CLI (boto3, awscli)    │
                     └──────────────────┬───────────────────┘
                                        │ HTTP/HTTPS :4566
                                        ▼
┌───────────────────────────────────────────────────────────────────────┐
│                    HTTP Server (Hypercorn/Twisted)                    │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │            LocalstackAwsGateway (Handler Chain)                 │ │
│  │                                                                 │ │
│  │  Request → Parse Service → Parse Protocol → Route → Dispatch   │ │
│  │     │                                                    │      │ │
│  │     └── 6 AWS protocols: query, json, rest-json,         │      │ │
│  │         rest-xml, ec2, smithy-rpc-v2-cbor                │      │ │
│  └──────────────────────────────────────────────┬──────────────────┘ │
└─────────────────────────────────────────────────┼────────────────────┘
                                                  │
              ┌───────────────────────────────────┼──────────────────┐
              │            Service Layer           │                  │
              │  ┌─────────────────────────────────┴───────────────┐ │
              │  │        Skeleton + DispatchTable                  │ │
              │  │  (Generated API stubs from botocore specs)      │ │
              │  └─────────────────────────────────────────────────┘ │
              │                     │                                 │
              │  ┌──────────┬──────┴──────┬──────────────┐          │
              │  │ Native   │ Moto        │ External     │          │
              │  │ Provider │ Fallback    │ Backend      │          │
              │  │ (S3,SQS) │ (IAM,EC2)  │ (DDB Local)  │          │
              │  └──────────┴─────────────┴──────────────┘          │
              │                                                      │
              │  State: AccountRegionBundle (per-account/region)     │
              │  Persistence: pickle/dill serialization to disk      │
              └──────────────────────────────────────────────────────┘
```

### 2.4 Key Architectural Components

| Component              | Technology              | Purpose                                    |
|------------------------|-------------------------|--------------------------------------------|
| HTTP Server            | Hypercorn (ASGI3)       | Accept incoming HTTP requests              |
| Gateway/Router         | rolo + werkzeug         | Handler chain, request routing             |
| Protocol Parser        | Custom (6 protocols)    | Parse AWS request formats                  |
| Response Serializer    | Custom (6 protocols)    | Serialize AWS response formats             |
| Service Framework      | ASF (custom)            | Provider registration, dispatch, lifecycle |
| API Stubs              | Generated from botocore | Type definitions for each AWS API          |
| State Management       | AccountRegionBundle     | Multi-account/region in-memory state       |
| Persistence            | pickle/dill             | Snapshot state to disk                     |
| Plugin System          | plux                    | Extension points, lazy loading             |
| Inter-service Comms    | Internal boto3 clients  | Service-to-service calls                   |
| Moto Integration       | moto-ext (fork)         | Fallback implementations for 15+ services  |

### 2.5 Moto Dependency Analysis

Services using `MotoFallbackDispatcher` (delegating unimplemented operations to moto):

**Heavy Moto dependency:** ACM, API Gateway (legacy+next-gen), CloudFormation (legacy),
Config, CloudWatch (v1), EC2, IAM, Logs, Route53, Route53Resolver, SES, SSM, STS,
StepFunctions, Support, SWF, Transcribe

**No Moto dependency (fully native):** S3, SQS, DynamoDB (uses DynamoDB Local instead),
Lambda, SNS, KMS, Events (EventBridge), Kinesis, Firehose

This is critical: **services with native implementations are the best candidates for
Rust migration** since they don't depend on moto's Python-only backend.

---

## 3. Feasibility Assessment

### 3.1 Is It Possible?

**Yes.** Each major concern has a viable solution:

| Concern                            | Viability | Solution                                                          |
|------------------------------------|-----------|-------------------------------------------------------------------|
| AWS protocol parsing (6 protocols) | High      | smithy-rs generates protocol handling from Smithy models          |
| API surface (265+ handlers)        | High      | smithy-rs auto-generates server stubs for all operations          |
| State management                   | High      | Rust has excellent concurrent data structures (dashmap, etc.)     |
| Plugin/extension system            | Medium    | Rust trait objects + dynamic loading (libloading) or WASM plugins |
| Moto replacement                   | Medium    | Must rewrite service logic natively (no moto equivalent in Rust)  |
| Inter-service communication        | High      | aws-sdk-rust provides typed clients for all services              |
| Persistence/snapshots              | High      | serde + bincode/messagepack for serialization                     |
| Python ecosystem migration         | High      | PyO3 enables incremental migration                                |
| Test parity                        | Medium    | Existing integration tests work against HTTP API (language-agnostic) |
| CloudFormation engine              | Low       | 41K LOC template engine -- extremely complex to rewrite           |
| StepFunctions ASL engine           | Low       | 39K LOC state machine engine -- extremely complex to rewrite      |

### 3.2 What Makes It Feasible (The Smithy Advantage)

The single most important enabler is **smithy-rs** and AWS's Smithy models:

```
AWS Smithy Models (JSON/Smithy IDL)
  ├── Define every operation, input/output shape, error, HTTP binding
  ├── Already exist for ALL AWS services
  │
  ├──[smithy-rs codegen-client]──→ aws-sdk-rust (client SDK)
  │
  └──[smithy-rs codegen-server]──→ Generated Rust Server Stubs
                                      │
                                      ├── HTTP routing (automatic)
                                      ├── Request deserialization (automatic)
                                      ├── Response serialization (automatic)
                                      ├── Error formatting (automatic)
                                      │
                                      └── Business logic trait (YOU IMPLEMENT THIS)
```

**This is exactly what LocalStack's ASF (AWS Service Framework) does in Python** --
but in Rust it would be auto-generated, type-safe, and orders of magnitude faster.

LocalStack currently:
1. Loads botocore JSON specs → generates Python API stubs
2. Creates Skeleton dispatcher from specs
3. Routes to @handler methods on Provider classes

Rust equivalent:
1. Feed Smithy models to smithy-rs codegen-server
2. Get auto-generated server with routing + (de)serialization
3. Implement trait methods for business logic

### 3.3 The PyO3 Bridge (Incremental Migration Enabler)

AWS has already built `aws-smithy-http-server-python` -- a crate that wraps Smithy-
generated Rust servers with PyO3 bindings, allowing Python functions as handlers:

```
HTTP Request
    │
    ▼
[Rust: hyper + tower + smithy routing + deserialization]  ← FAST
    │
    ▼
[Python via PyO3: business logic handler]                 ← Existing code
    │
    ▼
[Rust: serialization + HTTP response]                     ← FAST
```

This means we can:
1. Build the Rust HTTP framework
2. Keep existing Python service logic via PyO3
3. Gradually rewrite handlers in Rust, service by service
4. At no point is the system broken -- it's always functional

---

## 4. Rust Ecosystem Readiness

### 4.1 Key Crates

| Crate                             | Version  | Status     | Role                                      |
|-----------------------------------|----------|------------|-------------------------------------------|
| `smithy-rs` (codegen-server)      | ~0.66    | Active     | Generate server stubs from Smithy models  |
| `aws-smithy-http-server`          | 0.66.2   | Active     | Runtime for generated servers             |
| `aws-smithy-http-server-python`   | 0.66.1   | Active     | PyO3 bridge for Python handlers           |
| `aws-sdk-rust`                    | GA       | Production | Client SDK (for inter-service calls)      |
| `axum`                            | 0.8.x    | Production | HTTP framework (same tower/hyper stack)   |
| `tower`                           | 0.4.x    | Stable     | Middleware composition                    |
| `tokio`                           | 1.x      | Stable     | Async runtime                             |
| `PyO3`                            | 0.28.2   | Mature     | Python-Rust interop                       |
| `serde`                           | 1.x      | Stable     | Serialization framework                   |
| `dashmap`                         | 5.x      | Stable     | Concurrent hash maps (for state)          |
| `s3s`                             | 0.11.1   | Active     | Proves S3-in-Rust pattern works           |

### 4.2 Ecosystem Gaps

| Gap                                  | Severity | Mitigation                                    |
|--------------------------------------|----------|-----------------------------------------------|
| smithy-rs server API marked unstable | Medium   | Pin versions, track upstream closely           |
| No moto equivalent in Rust           | High     | Must implement service logic natively          |
| DynamoDB Local is Java               | Low      | Keep as subprocess (same as current approach)  |
| CloudFormation template engine       | High     | Keep in Python longest, or use WASM/PyO3       |
| Limited Smithy server examples       | Medium   | s3s project + AWS Pokemon example as reference |

### 4.3 Performance Expectations

Based on published benchmarks (FastAPI/Python vs Axum/Rust):

| Metric        | Python (current) | Rust (projected) | Improvement |
|---------------|------------------|-------------------|-------------|
| Throughput    | ~305 req/s       | ~9,700 req/s      | **32x**     |
| P50 Latency   | ~30 ms           | ~1 ms             | **30x**     |
| P99 Latency   | ~74 ms           | ~1.4 ms           | **53x**     |
| Memory (base) | ~83 MiB          | ~10 MiB           | **88% less**|
| Startup time  | ~5-15s           | <100ms            | **50-150x** |

Note: These are HTTP framework benchmarks. Actual service performance depends heavily on
business logic complexity. For in-memory services (SQS, SNS, KMS), gains will be
dramatic. For services proxying to external processes (DynamoDB Local), the improvement
will be primarily in the HTTP/serialization layer.

---

## 5. Architecture Design: RustStack

### 5.1 High-Level Architecture

```
                    ┌─────────────────────────────────┐
                    │   AWS SDK / CLI (any language)   │
                    └───────────────┬─────────────────┘
                                    │ HTTPS :4566
                                    ▼
┌───────────────────────────────────────────────────────────────────┐
│                     ruststack-gateway                             │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │  Hyper HTTP Server + Tower Middleware Stack                  │ │
│  │                                                             │ │
│  │  Layers: TLS → Logging → CORS → Auth → Metrics → Routing  │ │
│  └─────────────────────────┬───────────────────────────────────┘ │
│                             │                                     │
│  ┌─────────────────────────┴───────────────────────────────────┐ │
│  │  Service Router (Smithy-aware)                              │ │
│  │  - Parses signing name, target prefix, host, path           │ │
│  │  - Routes to correct service handler                        │ │
│  │  - Protocol detection (query/json/rest-json/rest-xml/ec2)   │ │
│  └─────────────────────────┬───────────────────────────────────┘ │
└─────────────────────────────┼────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────────┐
        │                     │                          │
        ▼                     ▼                          ▼
┌──────────────┐  ┌────────────────────┐  ┌──────────────────────┐
│ Rust-Native  │  │  PyO3-Bridged      │  │  External Backend    │
│ Services     │  │  Services          │  │  Services            │
│              │  │                    │  │                      │
│ - SQS        │  │  - CloudFormation  │  │  - DynamoDB          │
│ - S3         │  │  - StepFunctions   │  │    (DDB Local/Java)  │
│ - SNS        │  │  - API Gateway     │  │  - Elasticsearch     │
│ - KMS        │  │  - EC2             │  │  - OpenSearch        │
│ - Kinesis    │  │  - Lambda          │  │                      │
│ - STS        │  │  - IAM             │  │                      │
│ - Logs       │  │  - ...             │  │                      │
│ - SSM        │  │                    │  │                      │
└──────┬───────┘  └────────┬───────────┘  └──────────┬───────────┘
       │                   │                          │
       └───────────┬───────┘                          │
                   ▼                                  │
    ┌──────────────────────────────┐                  │
    │    State Management Layer    │◄─────────────────┘
    │                              │
    │  - AccountRegionStore<T>     │
    │  - DashMap-based concurrent  │
    │  - serde for persistence     │
    │  - Snapshot save/load        │
    └──────────────────────────────┘
```

### 5.2 Crate Organization

```
ruststack/
├── Cargo.toml                          # Workspace root
├── crates/
│   ├── ruststack-core/                 # Core types, config, errors
│   │   ├── src/
│   │   │   ├── config.rs               # Configuration (env vars, defaults)
│   │   │   ├── context.rs              # RequestContext (account, region, service)
│   │   │   ├── error.rs                # AWS error types (CommonServiceException)
│   │   │   ├── state.rs                # AccountRegionStore<T>
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── ruststack-gateway/              # HTTP gateway + routing
│   │   ├── src/
│   │   │   ├── server.rs               # Hyper server setup + TLS
│   │   │   ├── router.rs               # Service routing (signing name, path, host)
│   │   │   ├── middleware/
│   │   │   │   ├── cors.rs
│   │   │   │   ├── auth.rs             # SigV4 validation / pass-through
│   │   │   │   ├── logging.rs
│   │   │   │   └── metrics.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── ruststack-protocol/             # AWS protocol handling
│   │   ├── src/
│   │   │   ├── query.rs                # AWS Query protocol
│   │   │   ├── json.rs                 # AWS JSON protocol
│   │   │   ├── rest_json.rs            # REST-JSON protocol
│   │   │   ├── rest_xml.rs             # REST-XML protocol
│   │   │   ├── ec2.rs                  # EC2 query protocol
│   │   │   ├── cbor.rs                 # Smithy RPCv2 CBOR
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── ruststack-state/                # State management + persistence
│   │   ├── src/
│   │   │   ├── store.rs                # AccountRegionStore<T>
│   │   │   ├── persistence.rs          # Snapshot save/load (serde + bincode)
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── ruststack-bridge/               # PyO3 bridge for Python services
│   │   ├── src/
│   │   │   ├── python_handler.rs       # Call Python handlers from Rust
│   │   │   ├── state_bridge.rs         # Share state between Rust/Python
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── ruststack-codegen/              # Build-time: Smithy model → Rust code
│   │   ├── smithy-models/              # AWS Smithy model files
│   │   ├── src/
│   │   │   └── lib.rs                  # proc-macro or build.rs codegen
│   │   └── Cargo.toml
│   │
│   │── services/
│   │   ├── ruststack-sqs/              # SQS implementation
│   │   │   ├── src/
│   │   │   │   ├── provider.rs         # SQS business logic
│   │   │   │   ├── models.rs           # Queue, Message types
│   │   │   │   ├── store.rs            # SqsStore
│   │   │   │   └── lib.rs
│   │   │   └── Cargo.toml
│   │   ├── ruststack-s3/               # S3 implementation
│   │   ├── ruststack-sns/              # SNS implementation
│   │   ├── ruststack-kms/              # KMS implementation
│   │   ├── ruststack-kinesis/          # Kinesis implementation
│   │   ├── ruststack-sts/              # STS implementation
│   │   ├── ruststack-iam/              # IAM implementation
│   │   ├── ruststack-dynamodb/         # DynamoDB (proxy to DDB Local or native)
│   │   ├── ruststack-lambda/           # Lambda runtime management
│   │   └── .../
│   │
│   └── ruststack-cli/                  # CLI binary
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
│
├── codegen/                            # Smithy code generation tooling
│   ├── build.gradle.kts                # Smithy Gradle plugin
│   └── smithy-build.json               # Codegen configuration
│
├── tests/
│   ├── integration/                    # Integration tests (boto3-based, reused from LocalStack)
│   └── unit/                           # Rust unit tests
│
└── docker/
    └── Dockerfile                      # Multi-stage: builder → minimal runtime
```

### 5.3 Core Data Structures

#### AccountRegionStore (replaces Python's AccountRegionBundle)

```rust
use dashmap::DashMap;
use std::sync::Arc;

/// Thread-safe, multi-account, multi-region state store.
/// Replaces Python's AccountRegionBundle pattern.
pub struct AccountRegionStore<T: Default + Send + Sync> {
    inner: DashMap<(AccountId, Region), Arc<T>>,
}

impl<T: Default + Send + Sync> AccountRegionStore<T> {
    pub fn get_or_create(&self, account: &AccountId, region: &Region) -> Arc<T> {
        self.inner
            .entry((account.clone(), region.clone()))
            .or_insert_with(|| Arc::new(T::default()))
            .clone()
    }
}

/// Example: SQS Store
#[derive(Default)]
pub struct SqsStore {
    pub queues: DashMap<String, SqsQueue>,
    pub deleted_queues: DashMap<String, Instant>,
}
```

#### Service Provider Trait

```rust
/// Every service implements this trait (auto-generated by smithy-rs codegen).
/// The developer implements the business logic methods.
#[async_trait]
pub trait ServiceProvider: Send + Sync {
    fn service_name(&self) -> &str;

    /// Called when the service is first loaded
    async fn on_init(&self) -> Result<()> { Ok(()) }

    /// Called before the service starts accepting requests
    async fn on_start(&self) -> Result<()> { Ok(()) }

    /// Called on graceful shutdown
    async fn on_stop(&self) -> Result<()> { Ok(()) }

    /// State visitor for persistence
    fn accept_state_visitor(&self, visitor: &mut dyn StateVisitor) -> Result<()> {
        Ok(())
    }
}
```

### 5.4 Plugin / Extension System

```rust
/// Extension trait for third-party plugins
pub trait Extension: Send + Sync {
    fn name(&self) -> &str;

    /// Modify the router (add custom routes)
    fn update_routes(&self, _router: &mut Router) {}

    /// Add request middleware
    fn request_layer(&self) -> Option<Box<dyn Layer<...>>> { None }

    /// Add response middleware
    fn response_layer(&self) -> Option<Box<dyn Layer<...>>> { None }
}

// Extensions loaded via:
// 1. Compile-time feature flags (static linking)
// 2. Dynamic loading via libloading (shared libraries)
// 3. WASM plugins (future, via wasmtime)
```

---

## 6. Migration Strategy

### 6.1 Approach: Strangler Fig Pattern

Rather than a big-bang rewrite, use the **Strangler Fig** pattern:

```
Phase 0 (Now):     [========= Python LocalStack =========]
                    All traffic → Python

Phase 1 (Framework): [Rust Gateway] → [===== Python Services =====]
                      HTTP layer in Rust, services still Python via PyO3

Phase 2 (Services): [Rust Gateway] → [Rust SQS][Rust S3][Rust SNS] → [Python rest]
                     High-value services migrated to Rust

Phase 3 (Complete): [============= Rust RustStack ==============]
                     All services in Rust (Python bridge removed)
```

### 6.2 Service Migration Priority Matrix

Services ranked by: **Impact** (usage × performance sensitivity),
**Feasibility** (complexity, dependencies), and **Independence** (cross-service deps).

| Priority | Service       | LOC   | Moto Dep? | Rationale                                              |
|----------|---------------|-------|-----------|--------------------------------------------------------|
| **P0**   | SQS           | 4,842 | No        | Well-defined API, in-memory, perf-sensitive, no deps   |
| **P0**   | S3            | 12,350| No        | Most-used service, IO-heavy, benefits hugely from Rust |
| **P0**   | SNS           | 5,071 | No        | Pairs with SQS, moderate complexity, no moto           |
| **P0**   | KMS           | 3,129 | No        | Simple API, crypto benefits from Rust, no moto         |
| **P1**   | STS           | ~500  | Moto      | Foundation for auth, very small surface area           |
| **P1**   | Kinesis       | 983   | No        | Streaming perf-sensitive, small codebase               |
| **P1**   | CloudWatch    | 3,006 | Moto      | Moderate, used by many services                        |
| **P1**   | Logs          | 938   | Moto      | Small, pairs with CloudWatch                           |
| **P1**   | SSM           | 1,406 | Moto      | Small, widely used for config                          |
| **P2**   | SecretsManager| 1,664 | Moto      | Small, pairs with KMS                                  |
| **P2**   | DynamoDB      | 5,840 | No*       | *Proxies to Java DDB Local, complex streams            |
| **P2**   | Events        | 7,025 | No        | EventBridge, moderate complexity                       |
| **P2**   | IAM           | 3,014 | Moto      | Complex policy engine, heavy moto dependency           |
| **P3**   | Lambda        | 16,126| No        | Very complex (runtime mgmt, Docker, layers)            |
| **P3**   | EC2           | 4,243 | Moto      | Massive API surface, heavy moto fallback               |
| **P3**   | API Gateway   | 15,835| Moto      | Two implementations, very complex routing              |
| **P4**   | StepFunctions | 39,454| Moto      | ASL engine, enormous complexity                        |
| **P4**   | CloudFormation| 41,573| Moto      | Template engine, resource orchestration, massive       |
| **P4**   | EFS           | ~200  | N/A       | Minimal implementation in current LocalStack           |

*DynamoDB uses Java-based DynamoDB Local, not moto

### 6.3 Migration Decision Tree

```
For each service:

  Is it moto-dependent?
  ├── Yes → Does it have native handlers too?
  │         ├── Yes → Rewrite native handlers in Rust, implement moto-covered ops natively
  │         └── No  → Implement fully native in Rust (replaces moto coverage)
  │
  └── No → Rewrite provider logic directly in Rust
            └── Does it depend on external processes?
                ├── Yes (DDB Local, Kinesis Mock) → Keep external process, rewrite proxy layer
                └── No → Full native Rust implementation
```

---

## 7. Phase 1: Core Framework

**Duration estimate:** 3-4 months (2-3 senior Rust engineers)
**Goal:** Rust HTTP gateway that can proxy to existing Python LocalStack

### 7.1 Deliverables

1. **ruststack-gateway**: Hyper-based HTTP server with Tower middleware
   - TLS termination (rustls)
   - CORS handling
   - Request logging + metrics
   - AWS SigV4 signature parsing (pass-through, not validation)

2. **ruststack-protocol**: AWS protocol parsers/serializers
   - Ideally auto-generated from Smithy models via smithy-rs
   - Manual implementation as fallback for protocols smithy-rs doesn't cover

3. **Service Router**:
   - Parse service from: Authorization header, X-Amz-Target, Host, Path
   - Route to appropriate service handler
   - Support all 6 AWS protocols

4. **ruststack-state**: State management primitives
   - `AccountRegionStore<T>` with DashMap
   - Snapshot persistence (serde + bincode)
   - State reset mechanism

5. **ruststack-bridge**: PyO3 bridge
   - Call existing Python service providers from Rust gateway
   - Share RequestContext between Rust and Python
   - Async bridge (tokio ↔ asyncio via pyo3-async-runtimes)

6. **ruststack-cli**: Single binary entry point
   - `ruststack start` → start the server
   - `ruststack status` → health check
   - Environment variable configuration (same as LocalStack)

### 7.2 Validation Criteria

- [ ] All existing LocalStack integration tests pass via Rust gateway → Python bridge
- [ ] HTTP throughput ≥ 5x current LocalStack for passthrough requests
- [ ] Memory usage ≤ 50% of current LocalStack (for gateway layer)
- [ ] Startup time < 2 seconds (gateway only, Python services lazy-loaded)
- [ ] Docker image size reduction (no Python runtime in final phase)

### 7.3 Key Technical Decisions

**Smithy codegen vs manual protocol implementation:**

Option A: Use `smithy-rs codegen-server` to generate server stubs from AWS Smithy models.
- Pro: Auto-generated routing, parsing, serialization for ALL operations
- Pro: Type-safe, matches real AWS behavior exactly
- Con: smithy-rs server API is unstable, codegen toolchain is Kotlin/Gradle
- Con: May need patches for edge cases (as s3s project discovered)

Option B: Manual protocol implementation using the botocore JSON specs (like current LocalStack).
- Pro: Full control, no external codegen dependency
- Pro: Can handle LocalStack-specific extensions easily
- Con: Enormous manual effort for 6 protocols × hundreds of operations

**Recommendation:** Start with smithy-rs codegen for P0 services (SQS, S3, SNS, KMS).
Fall back to manual implementation only where codegen fails. The s3s project proves
this approach works for S3.

---

## 8. Phase 2: Priority Services

**Duration estimate:** 6-9 months (3-4 Rust engineers)
**Goal:** P0 and P1 services fully native in Rust

### 8.1 SQS (First Service - Template for All Others)

**Why first:** Clean API, moderate complexity, pure in-memory, no external dependencies,
well-defined behavior, good test coverage, high performance sensitivity.

**Implementation outline:**

```rust
// ruststack-sqs/src/models.rs

pub struct SqsQueue {
    pub name: String,
    pub arn: String,
    pub url: String,
    pub region: String,
    pub account_id: String,

    // Configuration
    pub visibility_timeout: Duration,
    pub message_retention_period: Duration,
    pub delay_seconds: Duration,
    pub max_message_size: usize,
    pub receive_message_wait_time: Duration,

    // FIFO
    pub fifo_queue: bool,
    pub content_based_deduplication: bool,
    pub deduplication_scope: DeduplicationScope,

    // DLQ
    pub redrive_policy: Option<RedrivePolicy>,
    pub redrive_allow_policy: Option<RedriveAllowPolicy>,

    // State
    pub messages: PriorityQueue<SqsMessage>,  // tokio-aware priority queue
    pub inflight: DashMap<String, SqsMessage>, // receipt_handle → message
    pub tags: DashMap<String, String>,
    pub attributes: QueueAttributes,
}

pub struct SqsMessage {
    pub message_id: String,
    pub body: String,
    pub md5_of_body: String,
    pub attributes: MessageAttributes,
    pub sent_timestamp: Instant,
    pub first_receive_timestamp: Option<Instant>,
    pub receive_count: u32,
    pub visibility_deadline: Option<Instant>,
    pub delay_until: Option<Instant>,
    pub sequence_number: Option<String>,  // FIFO only
    pub message_group_id: Option<String>, // FIFO only
    pub deduplication_id: Option<String>, // FIFO only
}
```

**Key implementation challenges:**
- Long-polling (`ReceiveMessage` with `WaitTimeSeconds`) → tokio::select with timeout
- Visibility timeout management → background task with tokio::interval
- FIFO ordering guarantees → per-group sequential processing
- Dead letter queue routing → cross-queue message movement
- Message deduplication → time-windowed dedup cache

### 8.2 S3

**Why:** Most-used AWS service, IO-heavy (streaming uploads/downloads benefit enormously
from Rust's zero-copy IO), complex but well-documented API.

**Implementation approach:**
- Use s3s crate as reference/foundation (already implements S3 trait from Smithy models)
- Implement storage backends: in-memory (ephemeral) and filesystem (persistent)
- Key complexities: multipart upload, versioning, presigned URLs, lifecycle rules,
  notifications, CORS, website hosting, bucket policies

**Expected storage trait:**
```rust
#[async_trait]
pub trait S3ObjectStore: Send + Sync {
    async fn get_object(&self, bucket: &str, key: &str, range: Option<Range>) -> Result<S3Object>;
    async fn put_object(&self, bucket: &str, key: &str, body: Bytes, metadata: ObjectMetadata) -> Result<PutObjectOutput>;
    async fn delete_object(&self, bucket: &str, key: &str, version: Option<&str>) -> Result<()>;
    async fn list_objects(&self, bucket: &str, prefix: Option<&str>, ...) -> Result<ListObjectsOutput>;
    async fn head_object(&self, bucket: &str, key: &str) -> Result<HeadObjectOutput>;
    // ... multipart operations
}
```

### 8.3 SNS

**Why:** Pairs naturally with SQS, moderate complexity, no external dependencies.

**Key challenges:**
- Fan-out to multiple subscription types (SQS, Lambda, HTTP, Email, SMS)
- Message filtering policies (JSON-based attribute matching)
- FIFO topics (paired with FIFO SQS queues)
- Platform applications (mobile push - can be stubbed initially)

### 8.4 KMS

**Why:** Small API surface, pure crypto operations, Rust's crypto libraries are excellent.

**Key implementation:**
- Key generation and storage (AES-256, RSA, ECC)
- Encrypt/Decrypt operations
- Key policies and grants
- Key rotation
- Use `ring` or `aws-lc-rs` for crypto primitives

### 8.5 Testing Strategy for Phase 2

**Critical insight:** LocalStack's existing integration tests are language-agnostic --
they use boto3 to make HTTP calls to localhost:4566. These tests can validate Rust
services without modification.

```
Existing test suite (Python/boto3)
         │
         │ HTTP requests to localhost:4566
         ▼
    ┌─────────────┐
    │ Rust Gateway │
    └──────┬──────┘
           │
    ┌──────┴──────┐
    │ Rust Service │  ← Being validated
    └─────────────┘
```

Steps:
1. Stand up Rust service on the same port
2. Run existing LocalStack test suite against it
3. Compare pass rates: must be ≥ current LocalStack
4. Use `TEST_TARGET=AWS_CLOUD` snapshot tests as the ground truth

---

## 9. Phase 3: Full Migration

**Duration estimate:** 12-18 months (ongoing)
**Goal:** Remove Python dependency entirely (or keep only for legacy/complex services)

### 9.1 Remaining Services

After P0/P1, migrate in order of value and complexity:
- **P2:** SecretsManager, DynamoDB (native impl replacing DDB Local), Events, IAM
- **P3:** Lambda (most complex -- Docker orchestration, runtime management), EC2, API Gateway
- **P4:** StepFunctions (ASL engine), CloudFormation (template engine)

### 9.2 CloudFormation and StepFunctions Strategy

These two services represent **80K+ LOC** of the most complex logic (template parsing,
resource dependency graphs, ASL state machine execution). Options:

**Option A: Keep in Python via PyO3 bridge permanently**
- Pragmatic, allows focusing Rust effort on performance-critical services
- Python is actually well-suited for this kind of complex orchestration logic

**Option B: WASM-based plugin**
- Compile complex engines to WASM, run via wasmtime
- Language-agnostic, could even support other languages in the future

**Option C: Full Rust rewrite (long-term)**
- Enormous effort but yields a fully unified codebase
- Could be driven by community contributions over time

**Recommendation:** Option A initially. These services are not typically performance
bottlenecks (they orchestrate other services). Revisit after all other services are
migrated.

### 9.3 Final Architecture

```
                        ┌───────────────────┐
                        │  ruststack binary  │  ~15-30 MiB
                        │  (single static)   │
                        └─────────┬─────────┘
                                  │
                    ┌─────────────┼──────────────┐
                    │             │               │
              ┌─────┴────┐ ┌─────┴────┐  ┌──────┴──────┐
              │ Gateway  │ │ Services │  │ State Mgmt  │
              │ (tower)  │ │ (native) │  │ (dashmap +  │
              │          │ │          │  │  bincode)   │
              └──────────┘ └──────────┘  └─────────────┘

Docker image: ~50-100 MiB (vs ~1.5 GiB current LocalStack)
Memory usage: ~50-200 MiB under load (vs ~500 MiB+ current)
Startup time: <1 second (vs ~10-30 seconds current)
```

---

## 10. Risk Analysis

### 10.1 Technical Risks

| Risk                                    | Probability | Impact | Mitigation                                     |
|-----------------------------------------|-------------|--------|-------------------------------------------------|
| smithy-rs server API breaking changes   | High        | Medium | Pin versions, maintain fork if needed            |
| Incomplete Smithy model coverage        | Medium      | Medium | Manual implementation for edge cases             |
| PyO3 async bridge performance overhead  | Medium      | Low    | Only temporary during migration                  |
| AWS protocol edge cases not in Smithy   | Medium      | High   | Extensive testing against real AWS + LocalStack  |
| Rust compilation times slow dev cycles  | High        | Medium | Workspace structure, incremental compilation     |
| Community contribution drop (Rust barrier) | High     | High   | Maintain Python bridge for community services    |

### 10.2 Organizational Risks

| Risk                                    | Probability | Impact | Mitigation                                     |
|-----------------------------------------|-------------|--------|-------------------------------------------------|
| Maintaining two codebases during migration | Certain  | High   | Strangler fig pattern, clear service ownership  |
| Feature parity regression               | Medium      | High   | Run existing test suite continuously             |
| Talent acquisition (Rust engineers)     | Medium      | Medium | Rust popularity growing, good hiring signal     |
| Community fragmentation                 | Medium      | High   | Clear migration path, backwards-compatible APIs |

### 10.3 Strategic Risks

| Risk                                    | Probability | Impact | Mitigation                                     |
|-----------------------------------------|-------------|--------|-------------------------------------------------|
| AWS changes Smithy models incompatibly  | Low         | Medium | Track AWS SDK releases, automated model updates |
| Alternative tools emerge (e.g., better moto) | Low    | Medium | Performance advantage is durable                |
| Scope creep (rewriting everything)      | High        | High   | Strict phase gates, P0-P4 prioritization        |

---

## 11. Estimated Effort

### 11.1 Phase Breakdown

| Phase | Scope                              | Engineers | Duration | Confidence |
|-------|-------------------------------------|-----------|----------|------------|
| 1     | Core framework + PyO3 bridge       | 2-3       | 3-4 mo   | High       |
| 2a    | SQS + SNS (first services)         | 2-3       | 2-3 mo   | High       |
| 2b    | S3 + KMS                           | 3-4       | 3-4 mo   | Medium     |
| 2c    | STS + Kinesis + CW + Logs + SSM    | 3-4       | 3-4 mo   | Medium     |
| 3a    | SecretsMgr + DynamoDB + Events     | 3-4       | 4-6 mo   | Medium     |
| 3b    | IAM + Lambda + EC2                 | 4-5       | 6-9 mo   | Low        |
| 3c    | API GW + SFn + CFn                 | 4-5       | 9-12 mo  | Low        |

**Total: ~30-42 months with 3-5 engineers average**
(or ~18-24 months with 6-8 engineers)

### 11.2 Milestone Checkpoints

| Milestone                                  | Target     | Go/No-Go Criteria                        |
|-------------------------------------------|------------|------------------------------------------|
| M1: Rust gateway proxying Python services | Month 4    | All existing tests pass                  |
| M2: First Rust-native service (SQS)       | Month 7    | SQS tests pass, 10x perf improvement    |
| M3: P0 services complete (SQS,S3,SNS,KMS)| Month 12   | P0 tests pass, Docker image < 200 MiB   |
| M4: P1 services complete                  | Month 18   | Majority of traffic on Rust services     |
| M5: Python bridge optional                | Month 24   | Python only needed for P4 services       |
| M6: Full Rust (or permanent hybrid)       | Month 36+  | Decision point: full Rust or keep hybrid |

---

## 12. Recommendation

### 12.1 Should You Do It?

**Yes, but incrementally.** The key factors:

1. **The Smithy ecosystem makes this realistic.** Without smithy-rs, this would be a
   multi-year effort to manually implement protocol handling for 48 services. With it,
   the protocol layer is auto-generated.

2. **The PyO3 bridge makes this safe.** You're never in a broken state. Services
   migrate one at a time, with the existing test suite validating every step.

3. **The performance gains are substantial.** 30-50x latency improvement and 8x memory
   reduction make Rust LocalStack viable for CI/CD pipelines where startup time and
   resource usage matter enormously.

4. **Docker image size matters.** Going from ~1.5 GiB to ~100 MiB dramatically improves
   CI/CD pipeline speed (image pull time is often the bottleneck).

### 12.2 Recommended Starting Point

**Start with Phase 1 + SQS as the proof-of-concept service.**

SQS is the ideal first service because:
- Clean, well-defined API (20 operations)
- Pure in-memory (no external dependencies)
- Performance-sensitive (long-polling, high-throughput)
- Good test coverage in existing suite
- No moto dependency (fully native in current LocalStack)
- Moderate complexity (not trivial, not overwhelming)

If SQS succeeds in Rust with ≥ feature parity and significant performance gains,
the approach is validated for all other services.

### 12.3 What NOT to Rewrite (Keep in Python)

Some components are better left in Python, at least initially:

- **CloudFormation template engine** (41K LOC, orchestration-heavy)
- **StepFunctions ASL engine** (39K LOC, state machine interpreter)
- **Extensions/plugins written by community** (preserve ecosystem)
- **CLI** (can stay as a thin Python wrapper calling the Rust binary)

### 12.4 EFS Specifically

EFS has minimal implementation in the current LocalStack codebase (~200 LOC). It would
be a trivially small Rust service. However, EFS's value is in its filesystem semantics
(NFS mount), which requires OS-level integration rather than just API emulation. A Rust
EFS would primarily be an API stub that creates the metadata -- actual NFS serving would
need additional infrastructure (e.g., NFS kernel server). Low priority but low effort.

---

## Appendix A: Key File References in Current LocalStack

| Component                    | Path                                                    |
|------------------------------|---------------------------------------------------------|
| Gateway entry point          | `localstack-core/localstack/aws/app.py`                |
| Handler chain                | `localstack-core/localstack/aws/handlers/`             |
| Protocol parser              | `localstack-core/localstack/aws/protocol/parser.py`    |
| Protocol serializer          | `localstack-core/localstack/aws/protocol/serializer.py`|
| Service router               | `localstack-core/localstack/aws/protocol/service_router.py` |
| Service providers            | `localstack-core/localstack/services/providers.py`     |
| ASF skeleton/dispatch        | `localstack-core/localstack/aws/skeleton.py`           |
| State management             | `localstack-core/localstack/services/stores.py`        |
| Moto integration             | `localstack-core/localstack/services/moto.py`          |
| Service lifecycle            | `localstack-core/localstack/services/plugins.py`       |
| Inter-service client factory | `localstack-core/localstack/aws/connect.py`            |
| Config                       | `localstack-core/localstack/config.py`                 |
| Runtime entry                | `localstack-core/localstack/runtime/main.py`           |
| Generated API stubs          | `localstack-core/localstack/aws/api/`                  |
| S3 provider                  | `localstack-core/localstack/services/s3/provider.py`   |
| SQS provider                 | `localstack-core/localstack/services/sqs/provider.py`  |
| DynamoDB provider            | `localstack-core/localstack/services/dynamodb/provider.py` |

## Appendix B: Relevant Rust Ecosystem Links

| Project                          | URL                                                    |
|----------------------------------|--------------------------------------------------------|
| smithy-rs (server codegen)       | https://github.com/smithy-lang/smithy-rs              |
| aws-smithy-http-server           | https://crates.io/crates/aws-smithy-http-server       |
| aws-smithy-http-server-python    | https://crates.io/crates/aws-smithy-http-server-python|
| aws-sdk-rust                     | https://github.com/awslabs/aws-sdk-rust               |
| s3s (S3 in Rust reference)       | https://github.com/Nugine/s3s                         |
| axum                             | https://github.com/tokio-rs/axum                      |
| tower                            | https://github.com/tower-rs/tower                     |
| PyO3                             | https://github.com/PyO3/pyo3                          |
| dashmap                          | https://github.com/xacrimon/dashmap                   |

## Appendix C: Performance Benchmark Sources

| Benchmark                    | Python       | Rust        | Improvement |
|------------------------------|-------------|-------------|-------------|
| HTTP throughput (req/s)      | 305         | 9,740       | 32x         |
| P50 latency (ms)            | 30          | 1.0         | 30x         |
| P99 latency (ms)            | 74          | 1.4         | 53x         |
| Peak memory (MiB)           | 83          | 10          | 8.3x        |
| CPU-bound tasks              | baseline    | ~60x faster | 60x         |

Sources: Luke Hsiao (FastAPI vs Axum), jonvet.com (Python vs Rust web servers)
