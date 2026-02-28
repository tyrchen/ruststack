# RustStack

A high-performance, LocalStack-compatible AWS service emulator written in Rust.

Currently implements **S3** with full protocol support — 70 operations generated from the AWS Smithy model, including versioning, multipart uploads, CORS, tagging, ACLs, object lock, encryption, lifecycle, and more.

## Features

- **Full S3 protocol** — 70 operations covering buckets, objects, multipart uploads, versioning, and bucket configuration
- **AWS SDK compatible** — Drop-in replacement for LocalStack S3; works with any AWS SDK or CLI
- **SigV4 authentication** — Optional AWS Signature Version 4 request verification
- **Virtual-hosted & path-style** addressing for bucket routing
- **In-memory storage** with automatic disk spillover for large objects (configurable threshold)
- **Smithy-driven codegen** — Types auto-generated from the official AWS S3 Smithy model
- **Tiny Docker image** — Static musl binary in a scratch container (~15 MB)
- **Graceful shutdown** and health check endpoints for container orchestration

## Quick Start

```bash
# Build and run
cargo run -p ruststack-s3-server

# Or use the Makefile
make run-s3
```

The server listens on `0.0.0.0:4566` by default. Point any AWS SDK or CLI at `http://localhost:4566`:

```bash
aws s3 --endpoint-url http://localhost:4566 mb s3://my-bucket
aws s3 --endpoint-url http://localhost:4566 cp file.txt s3://my-bucket/
aws s3 --endpoint-url http://localhost:4566 ls s3://my-bucket/
```

### Docker

```bash
docker build -t ruststack-s3 .
docker run -p 4566:4566 ruststack-s3
```

Multi-arch images (amd64/arm64) are published to `ghcr.io/tyrchen/ruststack-s3` on tagged releases.

## Configuration

All settings are controlled via environment variables, matching LocalStack conventions:

| Variable | Default | Description |
|----------|---------|-------------|
| `GATEWAY_LISTEN` | `0.0.0.0:4566` | Bind address and port |
| `S3_SKIP_SIGNATURE_VALIDATION` | `true` | Skip SigV4 request verification |
| `S3_VIRTUAL_HOSTING` | `true` | Enable virtual-hosted-style addressing |
| `S3_DOMAIN` | `s3.localhost.localstack.cloud` | Virtual hosting domain |
| `S3_MAX_MEMORY_OBJECT_SIZE` | `524288` | Max object size (bytes) kept in memory before disk spillover |
| `DEFAULT_REGION` | `us-east-1` | Default AWS region |
| `LOG_LEVEL` | `info` | Log level (`error`, `warn`, `info`, `debug`, `trace`) |
| `RUST_LOG` | | Fine-grained tracing filter (overrides `LOG_LEVEL`) |

## Supported Operations

<details>
<summary><b>Bucket operations (43)</b></summary>

| Category | Operations |
|----------|-----------|
| CRUD | CreateBucket, DeleteBucket, HeadBucket, ListBuckets, GetBucketLocation |
| Versioning | GetBucketVersioning, PutBucketVersioning |
| Encryption | GetBucketEncryption, PutBucketEncryption, DeleteBucketEncryption |
| CORS | GetBucketCors, PutBucketCors, DeleteBucketCors |
| Lifecycle | GetBucketLifecycleConfiguration, PutBucketLifecycleConfiguration, DeleteBucketLifecycle |
| Policy | GetBucketPolicy, PutBucketPolicy, DeleteBucketPolicy, GetBucketPolicyStatus |
| Tagging | GetBucketTagging, PutBucketTagging, DeleteBucketTagging |
| Notifications | GetBucketNotificationConfiguration, PutBucketNotificationConfiguration |
| Logging | GetBucketLogging, PutBucketLogging |
| Public Access | GetPublicAccessBlock, PutPublicAccessBlock, DeletePublicAccessBlock |
| Ownership | GetBucketOwnershipControls, PutBucketOwnershipControls, DeleteBucketOwnershipControls |
| Object Lock | GetObjectLockConfiguration, PutObjectLockConfiguration |
| Accelerate | GetBucketAccelerateConfiguration, PutBucketAccelerateConfiguration |
| Payment | GetBucketRequestPayment, PutBucketRequestPayment |
| Website | GetBucketWebsite, PutBucketWebsite, DeleteBucketWebsite |
| ACL | GetBucketAcl, PutBucketAcl |

</details>

<details>
<summary><b>Object operations (18)</b></summary>

| Category | Operations |
|----------|-----------|
| CRUD | PutObject, GetObject, HeadObject, DeleteObject, DeleteObjects, CopyObject |
| Tagging | GetObjectTagging, PutObjectTagging, DeleteObjectTagging |
| ACL | GetObjectAcl, PutObjectAcl |
| Retention | GetObjectRetention, PutObjectRetention |
| Legal Hold | GetObjectLegalHold, PutObjectLegalHold |
| Attributes | GetObjectAttributes |

</details>

<details>
<summary><b>Multipart upload operations (7)</b></summary>

CreateMultipartUpload, UploadPart, UploadPartCopy, CompleteMultipartUpload, AbortMultipartUpload, ListParts, ListMultipartUploads

</details>

<details>
<summary><b>List operations (3)</b></summary>

ListObjects, ListObjectsV2, ListObjectVersions

</details>

## Architecture

```
ruststack-s3-model   — S3 types auto-generated from AWS Smithy model (codegen/)
ruststack-s3-xml     — XML serialization/deserialization (quick-xml)
ruststack-s3-auth    — AWS SigV4 authentication
ruststack-s3-http    — HTTP routing, request/response conversion, hyper service
ruststack-s3-core    — Business logic, in-memory state, storage engine
ruststack-core       — Shared types, config, multi-account/region state
ruststack-s3-server  — Server binary with graceful shutdown
```

### Request Pipeline

```
HTTP Request
  → Health check interception
  → S3Router (path-style / virtual-hosted-style)
  → Body collection
  → SigV4 authentication (optional)
  → Operation dispatch (S3Handler trait)
  → Business logic (RustStackS3 provider)
  → XML/JSON response serialization
  → HTTP Response
```

## Development

**Prerequisites:** Rust 1.93+ (pinned in `rust-toolchain.toml`)

```bash
make build      # Compile all crates
make check      # cargo check --all-targets --all-features
make test       # Run unit tests (cargo nextest)
make fmt        # Format with cargo +nightly fmt
make clippy     # Lint with -D warnings
make audit      # Security vulnerability check
make deny       # License policy enforcement
make codegen    # Regenerate S3 model types from Smithy
```

### Integration Tests

Integration tests use the official AWS SDK for Rust against a running server:

```bash
# Terminal 1: start the server
make run-s3

# Terminal 2: run integration tests
cargo test -p ruststack-s3-integration -- --ignored
```

Tests cover buckets, objects, multipart uploads, versioning, CORS, error handling, and conditional requests.

### CI/CD

| Workflow | Trigger | What it does |
|----------|---------|-------------|
| `build.yml` | Push / PR | Format, lint, test, coverage |
| `integration.yml` | Push / PR | AWS SDK integration tests, MinIO Mint compatibility |
| `nightly.yml` | Daily 06:00 UTC | Ceph s3-tests compatibility suite |
| `release-docker.yml` | Version tags | Multi-arch Docker image to GHCR |

## License

This project is distributed under the terms of MIT.

See [LICENSE](LICENSE.md) for details.

Copyright 2025 Tyr Chen
