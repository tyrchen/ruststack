# RustStack S3: Rust-Native S3 Service for LocalStack

**Date:** 2026-02-26
**Status:** Draft / RFC
**Depends on:** [rust-rewrite-feasibility.md](./rust-rewrite-feasibility.md)
**Scope:** Replace LocalStack's Python S3 provider with a Rust implementation, ship it
as a standalone Docker image, and validate it in GitHub CI using LocalStack's existing
463 S3 tests.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Goals and Non-Goals](#2-goals-and-non-goals)
3. [Current S3 Implementation Analysis](#3-current-s3-implementation-analysis)
4. [Technical Design](#4-technical-design)
5. [Crate Structure](#5-crate-structure)
6. [Storage Backend Design](#6-storage-backend-design)
7. [Feature Implementation Plan](#7-feature-implementation-plan)
8. [CI/CD Pipeline Design](#8-cicd-pipeline-design)
9. [Testing Strategy](#9-testing-strategy)
10. [Docker Image Design](#10-docker-image-design)
11. [Integration with LocalStack Ecosystem](#11-integration-with-localstack-ecosystem)
12. [Implementation Phases](#12-implementation-phases)
13. [Risk Analysis](#13-risk-analysis)
14. [Open Questions](#14-open-questions)

---

## 1. Executive Summary

This spec describes building a Rust-native S3 service (`ruststack-s3`) that is
API-compatible with LocalStack's current Python S3 provider. The Rust implementation
will be validated in GitHub CI by running LocalStack's existing 463 S3 integration
tests against it.

**Key design decisions:**
- **Foundation:** Build on the `s3s` crate (v0.12), which provides a 96-operation S3
  trait, HTTP routing, XML serialization, and SigV4 auth -- all generated from AWS
  Smithy models.
- **What we build:** A custom storage backend implementing the `s3s::S3` trait, with
  in-memory storage, versioning, multipart uploads, presigned URLs, CORS, and
  event notifications.
- **What we reuse:** s3s handles HTTP server, request routing, XML (de)serialization,
  virtual-hosted/path-style addressing, and signature verification.
- **Docker image:** Static musl binary → scratch-based image (~20-30 MiB vs ~500+ MiB
  current Python S3 image).
- **CI:** New GitHub Actions workflow that builds the Rust binary, runs it in Docker,
  and executes the existing `tests/aws/services/s3/` test suite against it.

**Expected outcomes:**
- 30-50x lower request latency for S3 operations
- ~95% reduction in Docker image size
- Sub-second startup time (vs ~10s for Python S3 image)
- Feature parity with LocalStack S3 for core operations

---

## 2. Goals and Non-Goals

### Goals

1. **Feature parity** with LocalStack's S3 provider for the most commonly used
   operations (bucket CRUD, object CRUD, multipart uploads, versioning, presigned
   URLs, CORS, basic ACLs, tagging, encryption metadata, checksums).
2. **Drop-in replacement**: Same port (4566), same API, same virtual-hosted and
   path-style addressing. Existing `aws s3` CLI commands and SDK code work unchanged.
3. **CI validation**: Automated GitHub Actions workflow that builds the Rust S3 image,
   runs LocalStack's S3 test suite against it, and reports pass/fail alongside the
   existing Python S3 image tests.
4. **Multi-architecture**: linux/amd64 and linux/arm64 Docker images.
5. **Minimal image**: Target < 50 MiB Docker image (static musl binary).
6. **Self-contained**: No Python runtime, no JVM, no Node.js required.

### Non-Goals (for initial release)

1. **Full notification system**: S3 → SQS/SNS/Lambda/EventBridge notifications require
   those services to exist. Initial release will accept notification configurations but
   not dispatch events. (Phase 2 feature.)
2. **Cross-service integration**: No KMS key validation, no IAM policy evaluation, no
   CloudFormation resource provider.
3. **Persistence/snapshots**: Initial release is ephemeral (in-memory). State does not
   survive restarts. (Phase 2: serde + bincode snapshots.)
4. **Website hosting**: Static website hosting mode. (Phase 2.)
5. **Lifecycle rules execution**: Accept/store lifecycle configuration but don't execute
   expiration/transition. (Phase 2.)
6. **Select Object Content**: SQL-based object querying. (Phase 3 or never.)
7. **Replace the main LocalStack image**: This is a standalone S3-only image, not a
   replacement for the full LocalStack Docker image.

---

## 3. Current S3 Implementation Analysis

### 3.1 Codebase Metrics

| Component                  | File                    | Lines  |
|----------------------------|-------------------------|--------|
| Provider (business logic)  | `provider.py`           | 5,072  |
| Data models                | `models.py`             | 818    |
| Utilities                  | `utils.py`              | 1,194  |
| Notifications              | `notifications.py`      | 802    |
| Presigned URLs             | `presigned_url.py`      | 935    |
| CORS                       | `cors.py`               | 312    |
| Website hosting            | `website_hosting.py`    | 411    |
| Validation                 | `validation.py`         | 528    |
| Checksums                  | `checksums.py`          | 169    |
| Storage (abstract + impl)  | `storage/`              | ~800   |
| Constants, headers, codec  | various                 | ~420   |
| **Total**                  |                         | **~11,500** |

### 3.2 Operation Coverage

LocalStack's S3 provider implements **130+ operations**. Of these:
- **11 operations** use `@handler(expand=False)` for complex custom handling
- **~120 operations** override the auto-generated `S3Api` base class methods directly
- The `s3s` trait provides **96 operations** -- covering the vast majority of what
  LocalStack implements

### 3.3 Handler Complexity Matrix

| Handler              | Lines | Complexity | Key Challenge                              |
|----------------------|-------|------------|--------------------------------------------|
| `PutObject`          | 220   | Complex    | 40+ params, encryption, checksums, locks   |
| `GetObject`          | 180   | Complex    | Range requests, streaming, concurrency     |
| `CopyObject`         | 640   | Very High  | Conditional headers, metadata replace/copy |
| `HeadObject`         | 120   | Medium     | Like GetObject without body                |
| `CreateBucket`       | 100   | Medium     | Location constraint, global uniqueness     |
| `CreateMultipartUpload` | 130 | Medium    | Encryption, tags, metadata setup           |
| `UploadPart`         | 150   | Medium     | Checksums, size validation                 |
| `UploadPartCopy`     | 110   | Complex    | Like CopyObject for parts                  |
| `CompleteMultipartUpload` | 190 | Complex  | Part validation, assembly, checksums       |
| `GetObjectAttributes`| 100   | Medium     | Selective attribute return                 |
| `PutBucketAcl`       | 25    | Simple     | ACL parsing                                |
| `PutObjectAcl`       | 25    | Simple     | ACL parsing                                |

### 3.4 External Dependencies

The current Python S3 provider calls these other services:
- **KMS**: Validate encryption key IDs (can be stubbed with config flag)
- **SQS/SNS/Lambda/EventBridge**: Event notifications (deferred to Phase 2)
- **IAM**: Policy evaluation (implicit, not directly called)

### 3.5 What s3s Already Handles

| Feature                         | s3s provides? | Notes                                  |
|---------------------------------|---------------|----------------------------------------|
| HTTP server (hyper+tower)       | Yes           | HTTP/1.1 + HTTP/2                      |
| Virtual-hosted style addressing | Yes           | `S3Host` trait with domain config      |
| Path-style addressing           | Yes           | Automatic fallback                     |
| XML serialization/deser         | Yes           | From Smithy model, with S3 quirks      |
| SigV4 signature verification    | Yes           | `S3Auth` trait                         |
| Request routing (96 operations) | Yes           | Auto-generated from Smithy             |
| Streaming (upload/download)     | Yes           | `ByteStream` trait                     |
| Tower middleware integration    | Yes           | Standard tower::Layer support          |
| Error formatting                | Yes           | S3-standard XML error responses        |
| Multipart upload routing        | Yes           | All 5 multipart operations             |
| Bucket/Object CRUD routing      | Yes           | All standard operations                |

### 3.6 What We Must Build

| Feature                         | Scope   | Complexity |
|---------------------------------|---------|------------|
| In-memory storage backend       | Phase 1 | High       |
| Versioning (KeyStore/VersionedKeyStore) | Phase 1 | High |
| Multipart upload assembly       | Phase 1 | Medium     |
| CORS handling                   | Phase 1 | Medium     |
| Presigned URL enhancement       | Phase 1 | Medium     |
| Object Lock / WORM              | Phase 1 | Medium     |
| Checksum algorithms             | Phase 1 | Low        |
| Tagging                         | Phase 1 | Low        |
| Encryption metadata             | Phase 1 | Low        |
| ACL handling                    | Phase 1 | Low        |
| Lifecycle config storage        | Phase 1 | Low        |
| Notification dispatch           | Phase 2 | High       |
| Website hosting                 | Phase 2 | Medium     |
| Persistence (snapshot/restore)  | Phase 2 | Medium     |
| Replication config + execution  | Phase 3 | High       |

---

## 4. Technical Design

### 4.1 Architecture

```
                 ┌─────────────────────────────────┐
                 │    AWS SDK / CLI / boto3         │
                 └───────────────┬─────────────────┘
                                 │ HTTP :4566
                                 ▼
┌────────────────────────────────────────────────────────────┐
│                    s3s HTTP Layer                           │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  hyper Server + tower Middleware                      │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────┐  │  │
│  │  │  TLS     │→│  CORS    │→│  Logging │→│  Auth  │  │  │
│  │  │ (rustls) │ │ (custom) │ │ (tracing)│ │(S3Auth)│  │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  s3s Request Router                                   │  │
│  │  - Virtual-hosted / path-style bucket extraction      │  │
│  │  - HTTP method + URI + query → S3 operation           │  │
│  │  - XML deserialization → S3Request<Input>             │  │
│  └────────────────────────┬─────────────────────────────┘  │
└────────────────────────────┼───────────────────────────────┘
                             │
                             ▼
┌────────────────────────────────────────────────────────────┐
│              RustStackS3 (implements s3s::S3 trait)         │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Operation Dispatch (96 operations)                   │  │
│  │                                                       │  │
│  │  create_bucket() ──→ BucketManager                   │  │
│  │  put_object()    ──→ ObjectManager                   │  │
│  │  get_object()    ──→ ObjectManager + StorageBackend  │  │
│  │  copy_object()   ──→ ObjectManager (complex)         │  │
│  │  create_multipart_upload() ──→ MultipartManager      │  │
│  │  ...                                                  │  │
│  └──────────────────────────┬───────────────────────────┘  │
│                              │                              │
│  ┌──────────────────────────┴───────────────────────────┐  │
│  │                State Layer                            │  │
│  │                                                       │  │
│  │  AccountRegionStore<S3State>                          │  │
│  │  ├── buckets: DashMap<BucketName, S3Bucket>          │  │
│  │  └── global_bucket_map: DashMap<BucketName, AcctId>  │  │
│  │                                                       │  │
│  │  S3Bucket                                             │  │
│  │  ├── config (versioning, encryption, cors, lifecycle) │  │
│  │  ├── objects: KeyStore | VersionedKeyStore            │  │
│  │  ├── multiparts: DashMap<UploadId, Multipart>        │  │
│  │  └── tags, acl, notifications, policies               │  │
│  └──────────────────────────┬───────────────────────────┘  │
│                              │                              │
│  ┌──────────────────────────┴───────────────────────────┐  │
│  │              Storage Backend                          │  │
│  │                                                       │  │
│  │  InMemoryStore (default)                              │  │
│  │  ├── Small objects: Bytes (heap-allocated)            │  │
│  │  ├── Large objects: memory-mapped temp files          │  │
│  │  └── Streaming: tokio::io::AsyncRead                  │  │
│  │                                                       │  │
│  │  FsStore (optional, for persistence)                  │  │
│  │  ├── Objects: files on disk                           │  │
│  │  └── Metadata: JSON sidecar files                     │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────┘
```

### 4.2 Request Flow (Detailed)

```
1. HTTP request arrives at hyper server (:4566)
2. tower middleware chain:
   a. Optional TLS termination (rustls)
   b. Request logging (tracing)
   c. CORS preflight handling (OPTIONS → immediate response)
   d. Health check routing (/health, /_localstack/health)
3. s3s routing:
   a. Extract bucket from Host header (virtual-hosted) or path (path-style)
   b. Match HTTP method + path + query params → S3 operation
   c. Deserialize XML/query body → S3Request<OperationInput>
   d. If presigned URL: validate SigV4 signature
   e. If auth header: validate SigV4 (or pass-through for local dev)
4. Dispatch to RustStackS3::<operation>()
5. Business logic:
   a. Validate request (bucket exists, key valid, permissions)
   b. Execute operation (read/write state + storage)
   c. Construct S3Response<OperationOutput>
6. s3s serialization:
   a. Serialize response → XML/headers
   b. Stream body for large objects
7. CORS response headers added
8. HTTP response sent
```

### 4.3 Concurrency Model

```rust
// All state access is lock-free via DashMap or uses fine-grained RwLock
//
// Concurrency hierarchy:
//   1. AccountRegionStore: DashMap<(AccountId, Region), Arc<S3State>>
//   2. S3State.buckets: DashMap<BucketName, Arc<RwLock<S3Bucket>>>
//   3. S3Bucket.objects: RwLock<KeyStore> or RwLock<VersionedKeyStore>
//   4. Individual object data: Arc<StoredObject> (immutable once written)
//
// Key invariants:
//   - PutObject: write-lock on bucket.objects for the duration of key insertion
//   - GetObject: read-lock on bucket.objects for key lookup, then stream from
//                Arc<StoredObject> without holding the lock
//   - CopyObject: read-lock source bucket, write-lock dest bucket
//   - Multipart: DashMap per bucket for concurrent uploads, write-lock on
//                bucket.objects only during CompleteMultipartUpload
```

### 4.4 Configuration

Environment variables (same as LocalStack where applicable):

| Variable                          | Default          | Description                             |
|-----------------------------------|------------------|-----------------------------------------|
| `GATEWAY_LISTEN`                  | `0.0.0.0:4566`   | Bind address                            |
| `S3_VIRTUAL_HOSTING`              | `true`           | Enable virtual-hosted style             |
| `S3_DOMAIN`                       | `s3.localhost.localstack.cloud` | Domain for virtual hosting |
| `S3_SKIP_SIGNATURE_VALIDATION`    | `true`           | Skip SigV4 validation (dev mode)       |
| `S3_MAX_MEMORY_OBJECT_SIZE`       | `524288` (512KB) | Objects above this spill to temp files  |
| `PERSISTENCE`                     | `false`          | Enable state persistence (Phase 2)      |
| `DATA_DIR`                        | `/var/lib/localstack` | Persistence directory              |
| `LOG_LEVEL`                       | `info`           | Logging level (trace/debug/info/warn)   |
| `RUST_LOG`                        | (unset)          | Fine-grained tracing filter             |
| `DEFAULT_REGION`                  | `us-east-1`      | Default AWS region                      |

---

## 5. Crate Structure

```
ruststack-s3/
├── Cargo.toml                      # Workspace root
├── Cargo.lock
├── rust-toolchain.toml             # Pin Rust version (e.g., 1.83)
│
├── crates/
│   ├── ruststack-s3-server/        # Main binary crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs             # Entry point: parse config, start server
│   │       ├── config.rs           # Environment-based configuration
│   │       ├── health.rs           # Health check endpoints
│   │       └── server.rs           # hyper server setup + TLS + middleware
│   │
│   ├── ruststack-s3-core/          # Core S3 implementation (the s3s::S3 trait impl)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── provider.rs         # RustStackS3: implements s3s::S3 (96 operations)
│   │       │
│   │       ├── ops/                # Operation implementations (grouped by category)
│   │       │   ├── mod.rs
│   │       │   ├── bucket.rs       # create_bucket, delete_bucket, head_bucket, list_buckets
│   │       │   ├── bucket_config.rs # versioning, encryption, cors, lifecycle, logging, etc.
│   │       │   ├── object.rs       # put_object, get_object, head_object, delete_object, copy_object
│   │       │   ├── object_config.rs # tagging, acl, lock, retention, legal_hold
│   │       │   ├── multipart.rs    # create_multipart, upload_part, complete, abort, list_parts
│   │       │   ├── list.rs         # list_objects, list_objects_v2, list_object_versions
│   │       │   └── attributes.rs   # get_object_attributes
│   │       │
│   │       ├── state/              # State management
│   │       │   ├── mod.rs
│   │       │   ├── store.rs        # AccountRegionStore<S3State>
│   │       │   ├── bucket.rs       # S3Bucket struct + configuration
│   │       │   ├── object.rs       # S3Object, S3DeleteMarker, S3Part
│   │       │   ├── keystore.rs     # KeyStore + VersionedKeyStore
│   │       │   └── multipart.rs    # MultipartUpload state
│   │       │
│   │       ├── storage/            # Object data storage backends
│   │       │   ├── mod.rs
│   │       │   ├── traits.rs       # StorageBackend trait (async read/write/copy/delete)
│   │       │   ├── memory.rs       # InMemoryStorage (Bytes + tempfile spillover)
│   │       │   └── fs.rs           # FsStorage (optional, for persistence)
│   │       │
│   │       ├── validation.rs       # Request validation (bucket names, keys, ACLs, etc.)
│   │       ├── checksums.rs        # CRC32, CRC32C, SHA1, SHA256 computation
│   │       ├── cors.rs             # CORS rule matching + response headers
│   │       ├── error.rs            # S3-specific error types → s3s error mapping
│   │       └── utils.rs            # Shared utilities (ETags, version IDs, etc.)
│   │
│   └── ruststack-s3-notify/        # Notification system (Phase 2)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── dispatcher.rs       # Event dispatcher
│           ├── sqs.rs              # SQS notification target
│           ├── sns.rs              # SNS notification target
│           ├── lambda.rs           # Lambda invocation target
│           └── eventbridge.rs      # EventBridge target
│
├── docker/
│   ├── Dockerfile                  # Multi-stage build: rust → scratch
│   └── Dockerfile.test             # Test runner image (Python + boto3 + pytest)
│
├── .github/
│   └── workflows/
│       └── ruststack-s3.yml        # CI workflow
│
└── tests/
    ├── integration/                # Rust integration tests (basic)
    │   └── smoke_test.rs           # Create bucket, put/get object, delete
    └── conformance/                # Scripts to run LocalStack's S3 tests
        ├── run_localstack_tests.sh # Start Rust S3, run pytest, report
        └── pytest.ini              # Test config for running against Rust S3
```

### 5.1 Key Dependencies

```toml
# ruststack-s3-core/Cargo.toml
[dependencies]
s3s = "0.12"                   # S3 trait, routing, XML, auth
s3s-aws = "0.12"               # AWS SDK type conversions (optional)
tokio = { version = "1", features = ["full"] }
bytes = "1"
dashmap = "6"
parking_lot = "0.12"           # Fast RwLock/Mutex
uuid = { version = "1", features = ["v4"] }
chrono = "0.4"
md-5 = "0.10"
sha1 = "0.10"
sha2 = "0.10"
crc32fast = "1"
crc32c = "0.6"
base64 = "0.22"
tracing = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
quick-xml = "0.37"             # For any custom XML handling
thiserror = "2"
rand = "0.8"
tempfile = "3"                 # Large object spill-to-disk

# ruststack-s3-server/Cargo.toml
[dependencies]
ruststack-s3-core = { path = "../ruststack-s3-core" }
hyper = { version = "1", features = ["server", "http1", "http2"] }
hyper-util = { version = "0.1", features = ["tokio"] }
tower = { version = "0.5", features = ["util"] }
tower-http = { version = "0.6", features = ["cors", "trace"] }
rustls = "0.23"                # TLS (optional)
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
clap = { version = "4", features = ["derive", "env"] }
```

---

## 6. Storage Backend Design

### 6.1 Trait Definition

```rust
use bytes::Bytes;
use tokio::io::AsyncRead;

/// Handle to a stored object's data. Supports streaming reads.
pub trait StoredObjectRead: AsyncRead + Send + Unpin {
    /// Total size in bytes
    fn content_length(&self) -> u64;
    /// Precomputed ETag (MD5 hex)
    fn etag(&self) -> &str;
    /// Last modified timestamp (nanoseconds since epoch)
    fn last_modified_nanos(&self) -> u64;
}

/// Storage backend for S3 object data.
#[async_trait]
pub trait StorageBackend: Send + Sync + 'static {
    type Reader: StoredObjectRead;

    /// Write object data. Returns (etag, size, checksum).
    async fn write_object(
        &self,
        bucket: &str,
        key: &str,
        version_id: &str,
        body: impl AsyncRead + Send + Unpin,
        content_length: Option<u64>,
        checksum_algorithm: Option<ChecksumAlgorithm>,
    ) -> Result<WriteResult>;

    /// Open object for streaming read.
    async fn read_object(
        &self,
        bucket: &str,
        key: &str,
        version_id: &str,
        range: Option<ByteRange>,
    ) -> Result<Self::Reader>;

    /// Copy object data from source to destination.
    async fn copy_object(
        &self,
        src_bucket: &str,
        src_key: &str,
        src_version_id: &str,
        dst_bucket: &str,
        dst_key: &str,
        dst_version_id: &str,
        range: Option<ByteRange>,
    ) -> Result<WriteResult>;

    /// Delete object data.
    async fn delete_object(
        &self,
        bucket: &str,
        key: &str,
        version_id: &str,
    ) -> Result<()>;

    /// Write a multipart part.
    async fn write_part(
        &self,
        bucket: &str,
        upload_id: &str,
        part_number: u32,
        body: impl AsyncRead + Send + Unpin,
        checksum_algorithm: Option<ChecksumAlgorithm>,
    ) -> Result<WriteResult>;

    /// Assemble multipart parts into final object.
    async fn complete_multipart(
        &self,
        bucket: &str,
        upload_id: &str,
        dst_key: &str,
        dst_version_id: &str,
        parts: &[PartInfo],
    ) -> Result<WriteResult>;

    /// Delete all parts for an upload.
    async fn abort_multipart(
        &self,
        bucket: &str,
        upload_id: &str,
    ) -> Result<()>;

    /// Create bucket storage area.
    async fn create_bucket(&self, bucket: &str) -> Result<()>;

    /// Delete bucket storage area.
    async fn delete_bucket(&self, bucket: &str) -> Result<()>;

    /// Flush all pending writes (for persistence).
    async fn flush(&self) -> Result<()>;

    /// Reset all storage (for testing).
    async fn reset(&self) -> Result<()>;
}
```

### 6.2 In-Memory Storage Implementation

```rust
/// In-memory storage with automatic spill-to-disk for large objects.
pub struct InMemoryStorage {
    /// Object data: (bucket, key, version_id) → StoredData
    objects: DashMap<(String, String, String), StoredData>,
    /// Multipart parts: (bucket, upload_id, part_number) → StoredData
    parts: DashMap<(String, String, u32), StoredData>,
    /// Threshold above which objects spill to temp files
    max_memory_size: usize, // default: 512 KB
    /// Temp directory for spilled objects
    temp_dir: PathBuf,
}

enum StoredData {
    /// Small objects: kept entirely in memory
    InMemory {
        data: Bytes,
        etag: String,
        checksums: HashMap<ChecksumAlgorithm, String>,
        last_modified_nanos: u64,
    },
    /// Large objects: spilled to a temp file
    OnDisk {
        path: PathBuf,
        size: u64,
        etag: String,
        checksums: HashMap<ChecksumAlgorithm, String>,
        last_modified_nanos: u64,
    },
}
```

### 6.3 Object Data Flow

```
PutObject(body: AsyncRead)
  │
  ├─ body.len() ≤ 512KB? ──→ Read into Bytes, compute MD5 + checksum
  │                           Store as StoredData::InMemory
  │
  └─ body.len() > 512KB? ──→ Stream to tempfile in 64KB chunks
                              Compute MD5 + checksum incrementally
                              Store as StoredData::OnDisk

GetObject(range: Option<ByteRange>)
  │
  ├─ StoredData::InMemory ──→ Bytes::slice(range) → ByteStream
  │
  └─ StoredData::OnDisk   ──→ tokio::fs::File::open()
                               Seek to range.start
                               AsyncRead with limit → ByteStream
```

---

## 7. Feature Implementation Plan

### 7.1 Phase 1 Operations (MVP - Required for CI pass)

Target: Pass **≥80%** of LocalStack's 463 S3 tests.

#### Bucket Operations

| Operation | Priority | Complexity | Notes |
|-----------|----------|------------|-------|
| `CreateBucket` | P0 | Medium | Location constraint validation, global uniqueness |
| `DeleteBucket` | P0 | Low | Must be empty |
| `HeadBucket` | P0 | Low | Existence check |
| `ListBuckets` | P0 | Low | All buckets for account |
| `GetBucketLocation` | P0 | Low | Return region |
| `PutBucketVersioning` | P0 | Low | Enable/Suspend versioning |
| `GetBucketVersioning` | P0 | Low | Return status |
| `PutBucketEncryption` | P0 | Low | Store config (no actual encryption) |
| `GetBucketEncryption` | P0 | Low | Return config |
| `DeleteBucketEncryption` | P0 | Low | Remove config |
| `PutBucketTagging` | P0 | Low | Store tags |
| `GetBucketTagging` | P0 | Low | Return tags |
| `DeleteBucketTagging` | P0 | Low | Remove tags |
| `PutBucketCors` | P0 | Medium | Store + validate CORS rules |
| `GetBucketCors` | P0 | Low | Return rules |
| `DeleteBucketCors` | P0 | Low | Remove rules |
| `PutBucketPolicy` | P1 | Low | Store policy JSON |
| `GetBucketPolicy` | P1 | Low | Return policy |
| `DeleteBucketPolicy` | P1 | Low | Remove policy |
| `GetBucketPolicyStatus` | P1 | Low | Public access check |
| `PutBucketLifecycle` | P1 | Low | Store config (don't execute) |
| `GetBucketLifecycle` | P1 | Low | Return config |
| `DeleteBucketLifecycle` | P1 | Low | Remove config |
| `PutBucketNotificationConfiguration` | P1 | Low | Store config (don't dispatch) |
| `GetBucketNotificationConfiguration` | P1 | Low | Return config |
| `PutPublicAccessBlock` | P1 | Low | Store config |
| `GetPublicAccessBlock` | P1 | Low | Return config |
| `DeletePublicAccessBlock` | P1 | Low | Remove config |
| `PutBucketOwnershipControls` | P1 | Low | Store config |
| `GetBucketOwnershipControls` | P1 | Low | Return config |
| `DeleteBucketOwnershipControls` | P1 | Low | Remove config |
| `PutBucketAccelerateConfiguration` | P2 | Low | Store config |
| `GetBucketAccelerateConfiguration` | P2 | Low | Return config |
| `PutBucketRequestPayment` | P2 | Low | Store config |
| `GetBucketRequestPayment` | P2 | Low | Return config |
| `PutBucketLogging` | P2 | Low | Store config |
| `GetBucketLogging` | P2 | Low | Return config |

#### Object Operations

| Operation | Priority | Complexity | Notes |
|-----------|----------|------------|-------|
| `PutObject` | P0 | High | Streaming write, checksums, metadata, encryption headers, ACL, tagging, object lock |
| `GetObject` | P0 | High | Streaming read, range requests, conditional headers (If-Match, etc.), part number |
| `HeadObject` | P0 | Medium | Same as GetObject without body |
| `DeleteObject` | P0 | Medium | Delete markers for versioned buckets |
| `DeleteObjects` | P0 | Medium | Batch delete (up to 1000 keys) |
| `CopyObject` | P0 | Very High | Metadata replace/copy, conditional headers, cross-bucket, range copy |
| `ListObjects` | P0 | Medium | Prefix, delimiter, marker, max-keys |
| `ListObjectsV2` | P0 | Medium | Continuation token, start-after, fetch-owner |
| `ListObjectVersions` | P0 | Medium | Version listing with delete markers |
| `PostObject` | P1 | Medium | HTML form-based upload |
| `PutObjectTagging` | P0 | Low | Store tags |
| `GetObjectTagging` | P0 | Low | Return tags |
| `DeleteObjectTagging` | P0 | Low | Remove tags |
| `PutObjectAcl` | P1 | Low | Store ACL |
| `GetObjectAcl` | P1 | Low | Return ACL |
| `GetBucketAcl` | P1 | Low | Return bucket ACL |
| `PutBucketAcl` | P1 | Low | Store bucket ACL |
| `GetObjectAttributes` | P1 | Medium | Selective attribute return |
| `RestoreObject` | P2 | Low | Accept request, update restore status |
| `PutObjectLockConfiguration` | P1 | Low | Store config |
| `GetObjectLockConfiguration` | P1 | Low | Return config |
| `PutObjectRetention` | P1 | Low | Set retention |
| `GetObjectRetention` | P1 | Low | Return retention |
| `PutObjectLegalHold` | P1 | Low | Set legal hold |
| `GetObjectLegalHold` | P1 | Low | Return legal hold |

#### Multipart Operations

| Operation | Priority | Complexity | Notes |
|-----------|----------|------------|-------|
| `CreateMultipartUpload` | P0 | Medium | Generate upload ID, store metadata |
| `UploadPart` | P0 | Medium | Stream part data, compute checksums |
| `UploadPartCopy` | P0 | High | Copy from source object/range |
| `CompleteMultipartUpload` | P0 | High | Validate parts, assemble, compute composite checksums |
| `AbortMultipartUpload` | P0 | Low | Clean up parts |
| `ListParts` | P0 | Low | List uploaded parts |
| `ListMultipartUploads` | P1 | Medium | List in-progress uploads |

### 7.2 Phase 2 Operations (Full Feature Parity)

- Event notifications (SQS, SNS, Lambda, EventBridge dispatch)
- Website hosting (index/error documents, redirects, routing rules)
- Persistence (serde + bincode snapshot save/load)
- Lifecycle rule execution (TTL expiration)
- Replication configuration
- Bucket metrics/analytics/inventory (store + retrieve config)
- Intelligent tiering configuration

### 7.3 CORS Implementation

```rust
/// CORS middleware that wraps the s3s service.
/// Runs before s3s routing for OPTIONS preflight requests.
/// Runs after s3s for adding CORS headers to responses.
pub struct CorsMiddleware {
    /// Cache of bucket CORS rules, updated on PutBucketCors/DeleteBucketCors
    cors_index: Arc<CorsIndex>,
}

struct CorsIndex {
    /// bucket_name → CORS rules
    rules: DashMap<String, Vec<CorsRule>>,
}

impl CorsMiddleware {
    /// Handle OPTIONS preflight:
    /// 1. Extract bucket from Host/path
    /// 2. Look up CORS rules for bucket
    /// 3. Match Origin + Access-Control-Request-Method + Access-Control-Request-Headers
    /// 4. Return 200 with CORS headers or 403 AccessForbidden
    fn handle_preflight(&self, req: &Request) -> Option<Response> { ... }

    /// Add CORS headers to response:
    /// 1. Extract bucket from request
    /// 2. Match Origin against bucket's CORS rules
    /// 3. Add Access-Control-Allow-Origin, Expose-Headers, etc.
    fn add_cors_headers(&self, req: &Request, resp: &mut Response) { ... }
}
```

### 7.4 Presigned URL Strategy

s3s already handles SigV4 verification via the `S3Auth` trait. Our implementation:

```rust
struct RustStackAuth {
    skip_validation: bool, // S3_SKIP_SIGNATURE_VALIDATION
}

#[async_trait]
impl S3Auth for RustStackAuth {
    async fn get_secret_key(&self, access_key: &str) -> S3Result<SecretKey> {
        if self.skip_validation {
            // Return a dummy key that will match any signature
            // (s3s will skip verification)
            return Ok(SecretKey::from("test"));
        }
        // For real validation: look up the access key
        // Default: "test" → "test" (LocalStack default)
        match access_key {
            key => Ok(SecretKey::from("test")),
        }
    }
}
```

For presigned URLs, s3s handles the query parameter detection and signature
verification. We just need to provide the credential lookup.

---

## 8. CI/CD Pipeline Design

### 8.1 Workflow: `.github/workflows/ruststack-s3.yml`

```yaml
name: RustStack S3 / Build & Test

on:
  push:
    paths:
      - 'ruststack-s3/**'
      - '.github/workflows/ruststack-s3.yml'
      - 'tests/aws/services/s3/**'
    branches: [main]
  pull_request:
    paths:
      - 'ruststack-s3/**'
      - '.github/workflows/ruststack-s3.yml'
      - 'tests/aws/services/s3/**'
  workflow_dispatch:
    inputs:
      PYTEST_LOGLEVEL:
        type: choice
        description: Loglevel for PyTest
        options: [DEBUG, INFO, WARNING, ERROR, CRITICAL]
        default: WARNING

concurrency:
  group: ${{ github.workflow }}-${{ github.head_ref || github.run_id }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always
  PYTEST_LOGLEVEL: "${{ inputs.PYTEST_LOGLEVEL || 'WARNING' }}"

jobs:
  # ─── Job 1: Rust checks (fast feedback) ───────────────────────────
  rust-checks:
    name: "Rust lint & unit tests"
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: ruststack-s3
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: ruststack-s3 -> target

      - name: Format check
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Unit tests
        run: cargo test --all-features

  # ─── Job 2: Cross-compile for both architectures ──────────────────
  build:
    name: "Build (${{ matrix.arch }})"
    needs: rust-checks
    strategy:
      matrix:
        include:
          - arch: amd64
            target: x86_64-unknown-linux-musl
            runner: ubuntu-latest
          - arch: arm64
            target: aarch64-unknown-linux-musl
            runner: ubuntu-24.04-arm
    runs-on: ${{ matrix.runner }}
    defaults:
      run:
        working-directory: ruststack-s3
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: ruststack-s3 -> target
          key: ${{ matrix.target }}

      - name: Install musl tools
        run: sudo apt-get update && sudo apt-get install -y musl-tools

      - name: Build release binary
        run: cargo build --release --target ${{ matrix.target }}

      - name: Strip binary
        run: strip target/${{ matrix.target }}/release/ruststack-s3-server

      - name: Upload binary artifact
        uses: actions/upload-artifact@v4
        with:
          name: ruststack-s3-${{ matrix.arch }}
          path: ruststack-s3/target/${{ matrix.target }}/release/ruststack-s3-server
          retention-days: 1

  # ─── Job 3: Build Docker images ────────────────────────────────────
  docker:
    name: "Docker image (${{ matrix.arch }})"
    needs: build
    strategy:
      matrix:
        include:
          - arch: amd64
            platform: linux/amd64
          - arch: arm64
            platform: linux/arm64
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download binary
        uses: actions/download-artifact@v4
        with:
          name: ruststack-s3-${{ matrix.arch }}
          path: ruststack-s3/docker/bin/

      - name: Make binary executable
        run: chmod +x ruststack-s3/docker/bin/ruststack-s3-server

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build Docker image
        uses: docker/build-push-action@v5
        with:
          context: ruststack-s3/docker
          file: ruststack-s3/docker/Dockerfile
          platforms: ${{ matrix.platform }}
          tags: ruststack-s3:test-${{ matrix.arch }}
          outputs: type=docker,dest=/tmp/ruststack-s3-${{ matrix.arch }}.tar
          load: false

      - name: Upload Docker image
        uses: actions/upload-artifact@v4
        with:
          name: ruststack-s3-image-${{ matrix.arch }}
          path: /tmp/ruststack-s3-${{ matrix.arch }}.tar
          retention-days: 1

  # ─── Job 4: Run LocalStack S3 tests against Rust S3 ───────────────
  integration-test:
    name: "S3 integration tests (${{ matrix.arch }})"
    needs: docker
    strategy:
      matrix:
        include:
          - arch: amd64
            runner: ubuntu-latest
          - arch: arm64
            runner: ubuntu-24.04-arm
    runs-on: ${{ matrix.runner }}
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Download Rust S3 Docker image
        uses: actions/download-artifact@v4
        with:
          name: ruststack-s3-image-${{ matrix.arch }}
          path: /tmp/

      - name: Load Docker image
        run: docker load -i /tmp/ruststack-s3-${{ matrix.arch }}.tar

      - name: Start Rust S3 server
        run: |
          docker run -d \
            --name ruststack-s3 \
            -p 4566:4566 \
            -e S3_SKIP_SIGNATURE_VALIDATION=true \
            -e LOG_LEVEL=info \
            -e RUST_LOG=ruststack_s3=debug \
            ruststack-s3:test-${{ matrix.arch }}

      - name: Wait for server to be ready
        run: |
          for i in $(seq 1 30); do
            if curl -sf http://localhost:4566/_localstack/health > /dev/null 2>&1; then
              echo "Server is ready"
              exit 0
            fi
            sleep 1
          done
          echo "Server failed to start"
          docker logs ruststack-s3
          exit 1

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.13'

      - name: Install test dependencies
        run: |
          pip install boto3 pytest pytest-rerunfailures \
            localstack-snapshot requests httpx

      - name: Run S3 tests
        env:
          AWS_DEFAULT_REGION: us-east-1
          AWS_ACCESS_KEY_ID: test
          AWS_SECRET_ACCESS_KEY: test
          AWS_ENDPOINT_URL: http://localhost:4566
          TEST_TARGET: local
          TEST_S3_IMAGE: "true"
          PYTEST_ARGS: >-
            -x --timeout=60
            -o junit_family=legacy
            --junitxml=target/pytest-junit-ruststack-s3-${{ matrix.arch }}.xml
            -k "not (notification or lambda or eventbridge or sqs or sns)"
        run: |
          mkdir -p target
          python -m pytest tests/aws/services/s3/ \
            $PYTEST_ARGS \
            -v --tb=short 2>&1 | tee target/test-output.log

      - name: Dump server logs on failure
        if: failure()
        run: docker logs ruststack-s3

      - name: Stop Rust S3 server
        if: always()
        run: docker stop ruststack-s3 && docker rm ruststack-s3

      - name: Archive test results
        uses: actions/upload-artifact@v4
        if: success() || failure()
        with:
          name: test-results-ruststack-s3-${{ matrix.arch }}
          path: target/pytest-junit-ruststack-s3-${{ matrix.arch }}.xml
          retention-days: 30

  # ─── Job 5: Publish test results ───────────────────────────────────
  publish-test-results:
    name: "Publish RustStack S3 Test Results"
    needs: integration-test
    runs-on: ubuntu-latest
    if: success() || failure()
    permissions:
      checks: write
      pull-requests: write
      contents: read
      issues: read
    steps:
      - name: Download test results
        uses: actions/download-artifact@v4
        with:
          pattern: test-results-ruststack-s3-*
          merge-multiple: true

      - name: Publish test results
        uses: EnricoMi/publish-unit-test-result-action@v2
        with:
          files: pytest-junit-ruststack-s3-*.xml
          check_name: "RustStack S3 Test Results (AMD64 / ARM64)"
          action_fail_on_inconclusive: true

  # ─── Job 6: Comparison report ──────────────────────────────────────
  comparison:
    name: "Compare Python vs Rust S3"
    needs: integration-test
    if: success() || failure()
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download Rust S3 results
        uses: actions/download-artifact@v4
        with:
          name: test-results-ruststack-s3-amd64
          path: results/rust/

      # If Python S3 image tests ran in the same workflow, download those too
      - name: Generate comparison report
        run: |
          echo "## RustStack S3 vs Python S3 Test Comparison" > comparison.md
          echo "" >> comparison.md
          echo "### Rust S3 Results" >> comparison.md
          python3 -c "
          import xml.etree.ElementTree as ET
          tree = ET.parse('results/rust/pytest-junit-ruststack-s3-amd64.xml')
          root = tree.getroot()
          ts = root.find('.//testsuite') or root
          tests = ts.get('tests', '0')
          failures = ts.get('failures', '0')
          errors = ts.get('errors', '0')
          skipped = ts.get('skipped', '0')
          passed = int(tests) - int(failures) - int(errors) - int(skipped)
          total = int(tests)
          pct = (passed / total * 100) if total > 0 else 0
          print(f'| Metric | Value |')
          print(f'|--------|-------|')
          print(f'| Total tests | {tests} |')
          print(f'| Passed | {passed} |')
          print(f'| Failed | {failures} |')
          print(f'| Errors | {errors} |')
          print(f'| Skipped | {skipped} |')
          print(f'| **Pass rate** | **{pct:.1f}%** |')
          " >> comparison.md

      - name: Post comparison as PR comment
        if: github.event_name == 'pull_request'
        uses: marocchino/sticky-pull-request-comment@v2
        with:
          path: comparison.md
          header: ruststack-s3-comparison
```

### 8.2 CI Pipeline Visualization

```
┌──────────────────┐
│  rust-checks     │  ~2 min
│  (fmt+clippy+    │
│   unit tests)    │
└────────┬─────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌────────┐ ┌────────┐
│ build  │ │ build  │  ~5 min each
│ amd64  │ │ arm64  │
└───┬────┘ └───┬────┘
    │          │
    ▼          ▼
┌────────┐ ┌────────┐
│ docker │ │ docker │  ~2 min each
│ amd64  │ │ arm64  │
└───┬────┘ └───┬────┘
    │          │
    ▼          ▼
┌────────┐ ┌────────┐
│ integ  │ │ integ  │  ~15-20 min each
│ test   │ │ test   │
│ amd64  │ │ arm64  │
└───┬────┘ └───┬────┘
    │          │
    ▼          ▼
┌──────────────────┐
│ publish results  │  ~1 min
│ + comparison     │
└──────────────────┘

Total wall time: ~25-30 min
```

---

## 9. Testing Strategy

### 9.1 Test Layers

```
Layer 1: Rust Unit Tests (cargo test)
├── State management (KeyStore, VersionedKeyStore, AccountRegionStore)
├── Checksum computation (CRC32, SHA256, MD5)
├── Validation functions (bucket name, object key, ACL)
├── ETag generation (single object, multipart composite)
├── Version ID generation
└── CORS rule matching

Layer 2: Rust Integration Tests (cargo test --test)
├── Start server on random port
├── Use aws-sdk-s3 Rust client
├── Smoke tests: CreateBucket → PutObject → GetObject → DeleteObject
├── Multipart upload end-to-end
└── Versioning enable → put versions → list versions

Layer 3: LocalStack S3 Test Suite (pytest, Python/boto3)
├── 463 tests across 12 test files
├── Run against Rust S3 server on :4566
├── Same tests that validate Python S3 image
├── Tests excluded (require other services):
│   ├── test_s3_notifications_sqs.py (needs SQS)
│   ├── test_s3_notifications_sns.py (needs SNS)
│   ├── test_s3_notifications_lambda.py (needs Lambda)
│   └── test_s3_notifications_eventbridge.py (needs EventBridge)
├── ~350 tests expected to run (excluding notification tests)
└── Target: ≥80% pass rate for Phase 1, ≥95% for Phase 2
```

### 9.2 Test Execution Against Rust S3

The key insight is that LocalStack's S3 tests are **transport-level tests** -- they use
boto3 to make HTTP requests. They don't care whether the server is Python or Rust. To
run them against the Rust server:

```bash
# Start Rust S3 server
docker run -d -p 4566:4566 ruststack-s3:latest

# Run the existing LocalStack S3 tests
# The tests use AWS_ENDPOINT_URL=http://localhost:4566 to connect
AWS_DEFAULT_REGION=us-east-1 \
AWS_ACCESS_KEY_ID=test \
AWS_SECRET_ACCESS_KEY=test \
AWS_ENDPOINT_URL=http://localhost:4566 \
TEST_S3_IMAGE=true \
  pytest tests/aws/services/s3/ \
    -k "not (notification)" \
    -v --timeout=60
```

### 9.3 Test Compatibility Notes

Some LocalStack S3 tests may need adjustments:

| Issue | Tests Affected | Resolution |
|-------|---------------|------------|
| `TEST_S3_IMAGE` flag | Tests skip notification tests | Already handled |
| `_localstack/health` endpoint | Health checks | Implement health endpoint |
| `/_localstack/s3` internal APIs | Some dev tools | Implement or skip |
| Snapshot testing imports | `localstack_snapshot` | Must install as test dep |
| `conftest.py` fixtures | `s3_bucket`, `s3_create_bucket` | Must install localstack testing |
| `connect_to()` for inter-service | Notification tests | Already excluded |
| Custom CORS config env var | `DISABLE_CUSTOM_CORS_S3` | Implement env var |

### 9.4 Progressive Pass Rate Targets

| Milestone         | Target Pass Rate | Focus Area                               |
|-------------------|------------------|------------------------------------------|
| Week 2 (Alpha)    | 30-40%           | Basic CRUD: CreateBucket, Put/Get/Delete |
| Week 4 (Beta)     | 60-70%           | + Multipart, Versioning, List ops        |
| Week 6 (RC1)      | 80-85%           | + CopyObject, CORS, Presigned URLs       |
| Week 8 (RC2)      | 90-95%           | + Object Lock, ACLs, all config ops      |
| Phase 2           | ≥95%             | + Notifications, Website, Persistence    |

---

## 10. Docker Image Design

### 10.1 Dockerfile

```dockerfile
# ruststack-s3/docker/Dockerfile

# Minimal image: just the static binary
FROM scratch

# Copy the statically-linked musl binary
COPY bin/ruststack-s3-server /ruststack-s3-server

# Copy CA certificates for HTTPS (if needed for notifications in Phase 2)
COPY --from=alpine:3.19 /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Create required directories
# Note: scratch doesn't have mkdir, so we use VOLUME for runtime
VOLUME /var/lib/localstack
VOLUME /tmp/ruststack-s3

# Expose the S3 port (same as LocalStack)
EXPOSE 4566

# Health check
HEALTHCHECK --interval=5s --start-period=2s --retries=3 --timeout=3s \
  CMD ["/ruststack-s3-server", "health"]

# Labels
LABEL org.opencontainers.image.title="RustStack S3"
LABEL org.opencontainers.image.description="High-performance S3 emulator in Rust"

# Run as non-root (UID 1000 = localstack convention)
USER 1000

ENTRYPOINT ["/ruststack-s3-server"]
```

### 10.2 Image Size Comparison

| Image                          | Size     | Startup Time | Memory (idle) |
|--------------------------------|----------|--------------|---------------|
| LocalStack full                | ~1.5 GiB | ~15-30s      | ~250 MiB      |
| LocalStack S3-only (Python)    | ~500 MiB | ~10s         | ~120 MiB      |
| **RustStack S3 (projected)**   | **~20 MiB** | **<1s**   | **~5-10 MiB** |

### 10.3 Health Endpoint

```rust
// Implement as an s3s S3Route for non-S3 paths
impl S3Route for HealthRoute {
    fn is_match(&self, req: &Request) -> bool {
        let path = req.uri().path();
        path == "/_localstack/health"
            || path == "/health"
            || path == "/_ruststack/health"
    }

    async fn call(&self, _req: Request) -> Result<Response> {
        let body = serde_json::json!({
            "services": {
                "s3": "running"
            },
            "edition": "ruststack",
            "version": env!("CARGO_PKG_VERSION"),
        });
        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(body.to_string().into())?)
    }
}
```

---

## 11. Integration with LocalStack Ecosystem

### 11.1 Standalone Mode (Phase 1)

The Rust S3 server runs independently, replacing only the S3-only Docker image:

```
User code (boto3/awscli)
    │
    ▼ port 4566
┌───────────────┐
│ RustStack S3  │  ← standalone, no Python
└───────────────┘
```

### 11.2 Sidecar Mode (Phase 2)

Run alongside full LocalStack, handling only S3 while Python handles other services:

```
User code (boto3/awscli)
    │
    ▼ port 4566
┌──────────────────────────────────────────┐
│  LocalStack Gateway (Python)             │
│  ┌────────────────┐ ┌─────────────────┐  │
│  │ S3 → proxy to  │ │ Other services  │  │
│  │ RustStack :4577│ │ (SQS, SNS, etc) │  │
│  └───────┬────────┘ └─────────────────┘  │
└──────────┼───────────────────────────────┘
           │ port 4577
           ▼
┌──────────────────┐
│  RustStack S3    │
└──────────────────┘
```

This enables S3 notifications to work: the Python LocalStack can dispatch notifications
from S3 events by proxying through the Rust S3 server and forwarding events to
SQS/SNS/Lambda.

### 11.3 PyO3 Embedded Mode (Phase 3)

Embed the Rust S3 as a native Python module loaded by LocalStack:

```python
# In localstack's service provider
import ruststack_s3  # PyO3 native module

@aws_provider(api="s3", name="rust")
def s3_rust():
    provider = ruststack_s3.RustS3Provider(port=0)  # random port
    return Service.for_provider(provider)
```

---

## 12. Implementation Phases

### Phase 1: MVP (Weeks 1-8)

**Goal:** Pass ≥80% of LocalStack S3 tests, ship in CI.

| Week | Focus | Deliverables |
|------|-------|-------------|
| 1 | Project scaffolding | Cargo workspace, s3s integration, hyper server boots, health endpoint, CI skeleton |
| 2 | Storage + basic CRUD | InMemoryStorage, CreateBucket, PutObject, GetObject, HeadObject, DeleteObject, DeleteBucket |
| 3 | List + versioning | ListObjects, ListObjectsV2, PutBucketVersioning, versioned put/get/delete, ListObjectVersions, DeleteMarkers |
| 4 | Multipart uploads | CreateMultipartUpload, UploadPart, CompleteMultipartUpload, AbortMultipartUpload, ListParts, composite checksums |
| 5 | CopyObject + configs | CopyObject (full complexity), UploadPartCopy, all bucket config operations (encryption, tagging, policy, lifecycle, etc.) |
| 6 | CORS + presigned URLs | CorsMiddleware, preflight handling, S3Auth integration, presigned URL pass-through |
| 7 | Object Lock + ACL + polish | Object Lock (retention, legal hold), ACL handling, DeleteObjects batch, PostObject, error message parity |
| 8 | CI integration + stabilization | Full CI pipeline, test analysis, fix remaining failures, comparison reporting, Docker image optimization |

**Staffing:** 1-2 Rust engineers, full-time.

### Phase 2: Full Parity (Weeks 9-16)

| Week | Focus | Deliverables |
|------|-------|-------------|
| 9-10 | Notifications | SQS/SNS/Lambda/EventBridge event dispatch (requires HTTP client) |
| 11-12 | Website hosting | Static website mode, index/error docs, routing rules, redirects |
| 13-14 | Persistence | serde + bincode snapshot save/load, DATA_DIR support |
| 15-16 | Remaining features | Lifecycle execution (TTL), replication config, analytics/inventory/metrics configs |

**Target:** ≥95% test pass rate.

### Phase 3: Production Hardening (Weeks 17-24)

| Focus | Deliverables |
|-------|-------------|
| Performance | Benchmarking, profiling, optimization (zero-copy, buffer pooling) |
| Observability | Prometheus metrics endpoint, structured logging, request tracing |
| Security | Rate limiting, body size limits, TLS hardening |
| Sidecar mode | Proxy integration with full LocalStack |
| PyO3 module | Optional native Python module for LocalStack embedding |
| Documentation | API compatibility matrix, deployment guide, configuration reference |

---

## 13. Risk Analysis

### 13.1 Technical Risks

| Risk | Prob | Impact | Mitigation |
|------|------|--------|------------|
| s3s trait doesn't cover all needed operations | Medium | Medium | s3s covers 96/157 ops; remaining ~30 uncommon ops can return NotImplemented |
| S3 protocol edge cases (chunked encoding, etc.) | Medium | High | s3s handles most; test against AWS snapshot data for parity |
| Presigned URL handling gaps in s3s | Medium | Medium | s3s has known issue #438 with ports; patch or workaround |
| LocalStack test fixtures require full LocalStack | High | Medium | Install `localstack-core` as test dependency for fixtures; mock what we can't |
| Snapshot testing depends on `localstack_snapshot` | High | Low | Install as pip dependency in test runner |
| CORS middleware ordering vs s3s routing | Low | Medium | Implement as tower Layer wrapping s3s service |
| Memory usage for large objects | Low | Medium | Spill to tempfile above threshold (same as Python impl) |

### 13.2 Schedule Risks

| Risk | Prob | Impact | Mitigation |
|------|------|--------|------------|
| CopyObject complexity (640 LOC in Python) | High | Medium | Start early (Week 5), allocate extra time |
| Test fixture compatibility issues | High | Medium | Investigate in Week 1, build compatibility shim if needed |
| s3s version upgrade breaks API | Low | High | Pin version in Cargo.lock, track upstream |
| Cross-compilation issues (musl+crypto) | Medium | Low | Use native builds per-arch instead of cross |

### 13.3 Go/No-Go Criteria

| Milestone | Criterion | Fallback |
|-----------|-----------|----------|
| End of Week 2 | Basic CRUD works, 30% tests pass | Re-evaluate s3s choice |
| End of Week 4 | Multipart + versioning work, 60% pass | Narrow scope to CRUD-only image |
| End of Week 8 | ≥80% pass rate, CI green | Ship as "experimental" with known gaps |

---

## 14. Open Questions

### 14.1 Requires Decision Before Starting

1. **Repository location:** Should `ruststack-s3/` live inside the localstack repo
   (monorepo) or in a separate repository?
   - **Monorepo pro:** Shared CI, test co-location, single PR for test fixes
   - **Separate repo pro:** Independent release cadence, cleaner Rust workspace
   - **Recommendation:** Monorepo (inside `localstack/ruststack-s3/`), matching the
     existing pattern of `Dockerfile.s3` living in the main repo.

2. **Test runner approach:** How to run LocalStack's pytest tests against Rust S3?
   - **Option A:** Install `localstack-core` as a pip dependency in the test runner
     (needed for fixtures, markers, snapshot library).
   - **Option B:** Write a thin compatibility layer that provides just the needed
     fixtures without the full LocalStack dependency.
   - **Recommendation:** Option A is faster and ensures real parity. The test runner
     Docker image can include Python + localstack-core as test deps.

3. **Naming:** `ruststack-s3`? `localstack-s3-rs`? `localstack-s3-native`?
   - **Recommendation:** `ruststack-s3` -- clear, distinct from the Python impl,
     establishes the "ruststack" namespace for future Rust services.

### 14.2 Can Be Decided During Implementation

4. **s3s version pinning:** Use current 0.12 or track main branch?
5. **Logging format:** JSON structured logs (for CI parsing) or human-readable?
6. **TLS:** Include rustls by default or make it optional (most users use HTTP locally)?
7. **Multi-account support:** Full AccountRegionStore or simplified single-account for V1?
8. **Docker base image:** `scratch` (smallest, ~20 MiB) vs `distroless` (~30 MiB, has
   /tmp and CA certs) vs `alpine` (~25 MiB, has shell for debugging)?

---

## Appendix A: s3s S3 Trait Operations (96 total)

```
Bucket CRUD:          create_bucket, delete_bucket, head_bucket, list_buckets
Bucket Config:        get/put/delete_bucket_{acl,cors,encryption,lifecycle,
                      logging,notification,policy,replication,tagging,
                      versioning,website,accelerate,request_payment,
                      ownership_controls,public_access_block,
                      intelligent_tiering,analytics,metrics,inventory,
                      metadata_table,location}
Object CRUD:          put_object, get_object, head_object, delete_object,
                      delete_objects, copy_object, rename_object
Object Config:        get/put/delete_object_{acl,tagging,legal_hold,
                      lock_configuration,retention,attributes,torrent}
Object Misc:          restore_object, select_object_content,
                      write_get_object_response, post_object
Multipart:            create_multipart_upload, upload_part, upload_part_copy,
                      complete_multipart_upload, abort_multipart_upload,
                      list_parts, list_multipart_uploads
Listing:              list_objects, list_objects_v2, list_object_versions
Session:              create_session
```

## Appendix B: LocalStack S3 Test Files

| File | Tests | Runnable in Phase 1? |
|------|-------|---------------------|
| `test_s3.py` | 337 | Yes (most) |
| `test_s3_api.py` | 109 | Yes (except notification config tests) |
| `test_s3_list_operations.py` | 34 | Yes |
| `test_s3_cors.py` | 17 | Yes |
| `test_s3_concurrency.py` | 4 | Yes |
| `test_s3_preconditions.py` | 5 | Yes |
| `test_s3_notifications_sqs.py` | ~20 | No (needs SQS) |
| `test_s3_notifications_sns.py` | ~8 | No (needs SNS) |
| `test_s3_notifications_lambda.py` | ~5 | No (needs Lambda) |
| `test_s3_notifications_eventbridge.py` | ~5 | No (needs EventBridge) |
| **Total runnable Phase 1** | **~410** | |
| **Target pass (80%)** | **~328** | |

## Appendix C: Key s3s Crate References

| Crate | Purpose | Docs |
|-------|---------|------|
| `s3s` | Core S3 trait + routing + XML | https://docs.rs/s3s |
| `s3s-aws` | AWS SDK type conversions | https://docs.rs/s3s-aws |
| `s3s-fs` | Reference filesystem backend | https://docs.rs/s3s-fs |

## Appendix D: Existing CI Workflow Comparison

| Aspect | Python S3 Image CI | RustStack S3 CI |
|--------|-------------------|-----------------|
| Workflow | `aws-tests-s3-image.yml` | `ruststack-s3.yml` |
| Build time | ~5 min (Docker multi-stage) | ~7 min (Rust compile + Docker) |
| Test time | ~10 min | ~15-20 min (more thorough) |
| Image size | ~500 MiB | ~20-30 MiB |
| Test count | ~463 (all S3) | ~410 (minus notification tests) |
| Architectures | amd64 + arm64 | amd64 + arm64 |
| Test runner | Inside Docker container | External pytest → Docker S3 |
| Analytics | Tinybird | JUnit XML + PR comments |
