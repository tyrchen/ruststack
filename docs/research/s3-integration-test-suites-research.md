# S3 Integration Test Suites Research

**Date:** 2026-02-27
**Purpose:** Survey of available S3 compliance/integration test suites and recommendations for adoption in localstack-rs.

---

## Table of Contents

1. [Overview of Available Test Suites](#1-overview-of-available-test-suites)
2. [Ceph s3-tests](#2-ceph-s3-tests)
3. [MinIO Mint](#3-minio-mint)
4. [MSST-S3 (linux-kdevops)](#4-msst-s3-linux-kdevops)
5. [s3s Built-in Test Suite (s3s-e2e)](#5-s3s-built-in-test-suite-s3s-e2e)
6. [pytest + boto3 Approach](#6-pytest--boto3-approach)
7. [Rust aws-sdk-s3 Approach](#7-rust-aws-sdk-s3-approach)
8. [SNIA Cloud Object Storage Plugfest](#8-snia-cloud-object-storage-plugfest)
9. [How s3s Tests Itself (CI Deep Dive)](#9-how-s3s-tests-itself-ci-deep-dive)
10. [Comparison Matrix](#10-comparison-matrix)
11. [Recommendations for localstack-rs](#11-recommendations-for-localstack-rs)

---

## 1. Overview of Available Test Suites

There are several established S3 compatibility test suites in the ecosystem, each with different strengths:

| Test Suite | Language | Test Count | Docker Support | Active Maintenance |
|-----------|----------|------------|----------------|-------------------|
| Ceph s3-tests | Python (pytest/boto3) | ~400+ | No (native) | Yes (Ceph community) |
| MinIO Mint | Multi-SDK | ~100+ per SDK | Yes (Docker image) | Yes (MinIO) |
| MSST-S3 | Bash + multi-SDK | 618 | Yes (Docker Compose) | Yes (linux-kdevops) |
| s3s-e2e | Rust (aws-sdk-s3) | ~30+ | No (binary) | Yes (s3s project) |
| Custom pytest+boto3 | Python | Custom | N/A | N/A |
| Custom Rust aws-sdk-s3 | Rust | Custom | N/A | N/A |

---

## 2. Ceph s3-tests

**Repository:** https://github.com/ceph/s3-tests

### Overview

A set of unofficial Amazon AWS S3 compatibility tests originally developed for Ceph RADOS Gateway (RGW). This is one of the oldest and most comprehensive S3 test suites in existence.

### Technology

- Written in Python using **pytest** framework
- Uses both **boto2** (legacy) and **boto3** (current) client libraries
- Configuration-driven via `s3tests.conf` file

### Test Coverage

- **Bucket operations**: creation, listing, deletion, policies, ACLs, CORS, versioning, lifecycle
- **Object operations**: PUT, GET, DELETE, copy, multipart uploads, range requests
- **Access control**: ACLs, bucket policies, cross-account access, IAM policies
- **Advanced features**: S3 Select, server-side encryption (SSE-C), storage classes, website hosting, SNS integration
- **Authentication**: SigV4/SigV2 signatures, STS
- **Organization**: Tests tagged with pytest markers (e.g., `@pytest.mark.versioning`, `@pytest.mark.encryption`)
- **Files**: `test_s3.py` (main), `test_s3select.py`, `test_iam.py`, `test_sts.py`, `test_sns.py`

### How to Run Against Custom S3 Server

```ini
# s3tests.conf
[DEFAULT]
host = localhost
port = 4566
is_secure = False

[fixtures]
bucket prefix = test-

[s3 main]
access_key = test
secret_key = test
display_name = Test User

[s3 alt]
access_key = test2
secret_key = test2
display_name = Alt User
```

```bash
S3TEST_CONF=s3tests.conf tox -- -v s3tests_boto3/functional/test_s3.py
```

### Pros
- Most comprehensive S3 test suite available
- Battle-tested against multiple S3-compatible implementations
- Well-organized with pytest markers for selective test execution
- Covers edge cases and error conditions thoroughly

### Cons
- Python dependency management can be complex (uses tox)
- Some tests are Ceph-RGW-specific (marked with `fails_on_rgw` attribute)
- No Docker image -- requires local Python environment setup
- Tests assume two S3 users exist

---

## 3. MinIO Mint

**Repository:** https://github.com/minio/mint

### Overview

A containerized testing framework from MinIO that validates S3 correctness using multiple SDK clients.

### Technology

- Distributed as a **Docker/Podman image** (`minio/mint:edge`)
- Tests written in multiple languages/SDKs
- Results output as JSON log

### Included SDKs/Tools

| SDK/Tool | Language |
|----------|----------|
| awscli | Python |
| aws-sdk-go-v2 | Go |
| aws-sdk-java-v2 | Java |
| aws-sdk-php | PHP |
| aws-sdk-ruby | Ruby |
| minio-go | Go |
| minio-java | Java |
| minio-js | JavaScript |
| minio-py | Python |
| s3cmd | Python |
| s3select | - |
| mc (MinIO Client) | Go |

### Test Data

Pre-generated test files from 0 bytes to 129 MiB for various upload scenarios.

### How to Run Against Custom S3 Server

```bash
docker run \
  -e "SERVER_ENDPOINT=localhost:4566" \
  -e "ACCESS_KEY=test" \
  -e "SECRET_KEY=test" \
  --network host \
  -v /tmp/mint:/mint/log \
  minio/mint:edge
```

Test modes: `core` (basic) or `full` (comprehensive).

### Pros
- Easiest to run (single Docker command)
- Tests with multiple SDKs simultaneously (catches SDK-specific quirks)
- Well-suited for CI pipelines
- JSON-structured output for programmatic analysis

### Cons
- Heavyweight Docker image (includes multiple SDK runtimes)
- Some tests may be MinIO-specific
- Harder to debug individual test failures
- Less granular than Ceph s3-tests for operation-level coverage

---

## 4. MSST-S3 (linux-kdevops)

**Repository:** https://github.com/linux-kdevops/msst-s3

### Overview

A newer, comprehensive interoperability testing framework designed to validate S3 API compatibility across different storage implementations. Originally ported 592 tests from the versitygw project.

### Technology

- Shell-based test runner with multi-SDK support
- Makefile-driven with `menuconfig` for configuration
- Docker Compose files for spinning up various backends

### Test Coverage

- **618 comprehensive S3 API tests** across 75 test files
- Basic CRUD, multipart uploads, versioning, ACL, checksums, error handling
- 94.2% pass rate on MinIO (100% on critical tests)

### Pre-configured Backend Profiles

- AWS S3, MinIO (local), basic setup
- Docker Compose configs for: MinIO, RustFS, Ceph, LocalStack, Garage, SeaweedFS

### How to Run Against Custom S3 Server

```bash
make menuconfig        # Interactive configuration
# or
make defconfig-basic   # Use a preset
make test              # Run all tests
make test GROUP=multipart  # Specific category
make test TEST=001         # Single test
```

### Pros
- Largest test count (618 tests)
- Multi-backend comparison built-in
- Already has a LocalStack Docker Compose profile
- Shell-based runner is dependency-light
- Production validation suite included

### Cons
- Relatively new project (less battle-tested)
- Shell-based tests can be harder to maintain
- Ported from versitygw -- may have biases

---

## 5. s3s Built-in Test Suite (s3s-e2e)

**Repository:** https://github.com/Nugine/s3s (crates `s3s-test` and `s3s-e2e`)

### Overview

The s3s project includes its own Rust-native end-to-end test framework split across two crates.

### Architecture

**s3s-test** (framework crate):
- Custom async test runner (not using `#[test]` or `#[tokio::test]`)
- Hierarchical organization: Suites -> Fixtures -> Cases
- `TestSuite` trait: per-suite setup/teardown
- `TestFixture` trait: per-test-group setup/teardown
- `TestCase` trait: individual test execution
- CLI with `--filter` (regex), `--list`, and `--json` report output
- Color-coded pass/fail output with timing

**s3s-e2e** (test implementation crate):
- Uses `aws-sdk-s3` and `aws-sdk-sts` as S3 clients
- Organized into `basic` and `advanced` modules
- Compiled as a standalone binary

### Test Coverage -- Basic Suite

| Test | Operations Covered |
|------|-------------------|
| ListBuckets | Bucket enumeration |
| ListObjects | Object listing with Unicode |
| GetObject | Object retrieval and content verification |
| DeleteObject | Object removal verification |
| HeadBucket/HeadObject | Metadata retrieval |
| PutObject (tiny) | Empty and minimal uploads |
| PutObject (metadata) | Custom metadata, content-type |
| PutObject (non-ASCII metadata) | Unicode metadata, RFC2047 |
| PutObject (larger) | 1KB+ uploads |
| PutObject (checksum) | CRC32, CRC32C, SHA1, SHA256, CRC64NVME |
| PutObject (content checksum) | MD5 integrity success/failure |
| CopyObject | Object duplication |

### Test Coverage -- Advanced Suite

| Test | Operations Covered |
|------|-------------------|
| STS | assume_role, temporary credentials |
| Multipart Upload | 5MB+ chunks, CRC32 checksum, completion |
| Object Tagging | Put/Get tags |
| List Pagination | max_keys, continuation_token |
| Presigned URLs | PUT/GET with reqwest HTTP client |

### How to Run

The binary reads AWS SDK configuration from environment variables:

```bash
export AWS_ACCESS_KEY_ID=test
export AWS_SECRET_ACCESS_KEY=test
export AWS_ENDPOINT_URL=http://localhost:4566
export AWS_REGION=us-east-1

./s3s-e2e --filter "basic/.*"
./s3s-e2e --list
./s3s-e2e --json results.json
```

### Pros
- Written in Rust -- native to our project's ecosystem
- Uses `aws-sdk-s3` (same client we would use for real testing)
- Lightweight, fast execution
- Clean hierarchical test organization
- Directly tests what s3s considers important for compliance

### Cons
- Smaller test count (~30 tests) compared to Ceph or MSST-S3
- Tightly coupled to s3s project conventions
- No Docker packaging
- Limited coverage of error cases and edge conditions

---

## 6. pytest + boto3 Approach

### Overview

The most common approach seen across the ecosystem for integration testing S3-compatible services. Used by:
- Ceph s3-tests (primary approach)
- LocalStack's own internal tests
- `pytest-localstack` plugin ecosystem
- s3s project's regression tests

### LocalStack-specific Tooling

**pytest-localstack plugin** (https://github.com/mintel/pytest-localstack):

```python
import pytest_localstack

localstack = pytest_localstack.patch_fixture(
    services=["s3"],
    scope="module",
)

def test_s3_bucket_creation(localstack):
    client = localstack.session.client("s3")
    client.create_bucket(Bucket="test-bucket")
    response = client.list_buckets()
    assert any(b["Name"] == "test-bucket" for b in response["Buckets"])
```

**Standard boto3 client approach** (without plugin):

```python
import boto3

s3 = boto3.client(
    "s3",
    endpoint_url="http://localhost:4566",
    region_name="us-east-1",
    aws_access_key_id="test",
    aws_secret_access_key="test",
)
```

### GitHub Actions Integration

LocalStack provides an official GitHub Action:

```yaml
- name: Start LocalStack
  uses: localstack/setup-localstack@v2
  with:
    image-tag: 'latest'

- name: Run S3 Tests
  run: pytest tests/integration/
```

### Prevalence

Based on GitHub search and ecosystem analysis, **pytest + boto3 is significantly more common** than Rust aws-sdk-s3 for integration testing of S3-compatible services. This is because:
- boto3 is the most widely-used S3 client library
- Python test infrastructure is mature and well-documented
- Most S3-compatible projects originate from the Python/Go ecosystems

---

## 7. Rust aws-sdk-s3 Approach

### Overview

Using the official AWS SDK for Rust to write integration tests. This is what s3s-e2e does.

### Setup

```rust
use aws_sdk_s3::Client;
use aws_config::BehaviorVersion;

async fn create_test_client() -> Client {
    let config = aws_config::defaults(BehaviorVersion::latest())
        .endpoint_url("http://localhost:4566")
        .region(aws_config::Region::new("us-east-1"))
        .credentials_provider(
            aws_credential_types::Credentials::new("test", "test", None, None, "test")
        )
        .load()
        .await;

    Client::new(&config)
}
```

### Testcontainers for Rust

The `testcontainers` Rust crate supports LocalStack:

```rust
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::localstack::LocalStack;

let container = LocalStack::default().start().await?;
let endpoint = format!("http://localhost:{}", container.get_host_port(4566).await?);
```

### Prevalence

Less common than pytest+boto3 for S3 integration testing, but growing. Used primarily by:
- s3s project (s3s-e2e crate)
- Rust projects needing S3 integration tests
- Projects that want compile-time type safety in tests

---

## 8. SNIA Cloud Object Storage Plugfest

**Website:** https://www.snia.org/node/13718

### Overview

An industry-driven initiative where multiple vendors (Dell, Google, IBM, Microsoft, NetApp, VAST Data, Versity) collaborate on S3 interoperability testing. The SNIA Cloud Storage Technologies Community hosts regular plugfests.

### Key Findings

- Most recent plugfest: April 2025 (Denver)
- Next scheduled: September 2025 (Santa Clara, co-located with SDC'25)
- Goal: create and publish COS interoperability test software suites
- Community-identified issues: ambiguities in protocol options, access control mechanisms, missing/incorrect headers, unsupported API calls

### Relevance

The SNIA plugfest effort validates that S3 compatibility testing is an active industry concern. Their findings about protocol ambiguities should inform our test strategy, but their test suites are not yet publicly available as a standalone tool.

---

## 9. How s3s Tests Itself (CI Deep Dive)

The s3s project's CI configuration is highly informative because it uses **three different test suites** to validate its implementation:

### CI Pipeline (from `.github/workflows/ci.yml`)

1. **Rust unit/integration tests**: `cargo test` across MSRV (1.88.0), stable, and nightly
2. **Clippy + rustfmt**: Standard Rust linting
3. **Cross-platform**: Tests on Windows and macOS
4. **WASM**: Tests in wasm-pack
5. **Code coverage**: via `cargo llvm-cov`

### E2E Tests (non-PR events only -- merge groups and scheduled runs)

**Test 1: Mint against s3s-proxy + MinIO**
```bash
# scripts/e2e-mint.sh
./scripts/s3s-proxy.sh > target/s3s-proxy.log &
sleep 3s
./scripts/mint.sh | tee target/mint.log

# scripts/mint.sh
docker run \
  -e "SERVER_ENDPOINT=localhost:8014" \
  -e "ACCESS_KEY=minioadmin" \
  -e "SECRET_KEY=minioadmin" \
  --network host \
  -v /tmp/mint:/mint/log \
  minio/mint:edge
```

**Test 2: s3s-e2e against s3s-fs**
- Starts `s3s-fs` binary as local S3 server
- Runs `s3s-e2e` binary against it

**Test 3: Ceph s3-tests against s3s-fs**
- Clones `ceph/s3-tests` repository
- Caches Python dependencies by requirements hash
- Runs `./scripts/e2e-s3tests.sh`

**Test 4: boto3 regression tests**
- Runs specific Python test files from `tests/boto3/`

### Key Takeaway

The s3s project considers **all three** external test suites necessary for thorough S3 compliance validation. No single suite is sufficient on its own. The layered approach catches different categories of issues.

---

## 10. Comparison Matrix

| Criteria | Ceph s3-tests | MinIO Mint | MSST-S3 | s3s-e2e | Custom pytest | Custom Rust |
|----------|:------------:|:----------:|:-------:|:-------:|:------------:|:-----------:|
| **Test breadth** | Excellent | Good | Excellent | Basic | Varies | Varies |
| **Setup complexity** | Medium | Low | Medium | Low | Low | Low |
| **CI integration** | Medium | Easy | Medium | Easy | Easy | Easy |
| **Docker support** | No | Yes | Yes | No | N/A | N/A |
| **Multi-SDK coverage** | No (boto3 only) | Yes (12 SDKs) | Yes (multi-SDK) | No (aws-sdk-s3) | No | No |
| **Error case coverage** | Excellent | Good | Good | Basic | Varies | Varies |
| **Active maintenance** | Yes | Yes | Yes | Yes | N/A | N/A |
| **Rust ecosystem native** | No | No | No | Yes | No | Yes |
| **Customizability** | Good | Limited | Good | Good | Full | Full |
| **Operation coverage** | ~400+ tests | ~100+ per SDK | 618 tests | ~30 tests | Custom | Custom |
| **Used by s3s project** | Yes | Yes | No | Yes | Yes | N/A |

---

## 11. Recommendations for localstack-rs

### Recommended Multi-Layer Testing Strategy

Following the s3s project's proven approach, adopt a **tiered testing strategy**:

#### Tier 1: Rust Native Tests (Must Have -- Day 1)

**Approach**: Write integration tests using `aws-sdk-s3` in Rust, either:
- (a) Adopt `s3s-test` + `s3s-e2e` framework directly (if our project builds on s3s), or
- (b) Write custom integration tests using `aws-sdk-s3` + `tokio::test`

**Rationale**: These run as part of `cargo test`, provide compile-time type safety, catch Rust-specific issues immediately, and are the fastest feedback loop. Since we are building on s3s, reusing s3s-e2e or writing tests in the same style makes the most sense.

**Coverage target**: Core CRUD operations, multipart upload, presigned URLs, authentication, error responses.

#### Tier 2: MinIO Mint (Must Have -- Early Development)

**Approach**: Run Mint Docker container against our server in CI.

```yaml
# GitHub Actions
- name: Run Mint Tests
  run: |
    docker run \
      -e "SERVER_ENDPOINT=localhost:4566" \
      -e "ACCESS_KEY=test" \
      -e "SECRET_KEY=test" \
      --network host \
      -v /tmp/mint:/mint/log \
      minio/mint:edge
```

**Rationale**: Lowest setup cost, tests with multiple SDKs simultaneously (catches serialization bugs that single-SDK tests miss), well-proven in s3s CI. This is the best "bang for buck" external test suite.

**When to add**: As soon as core CRUD operations are implemented.

#### Tier 3: Ceph s3-tests (Should Have -- Mid Development)

**Approach**: Clone ceph/s3-tests and run the boto3 test suite against our server.

**Rationale**: Most comprehensive S3 test suite available, covers edge cases and error conditions that Mint does not. The s3s project uses this as its most thorough external validation. Catches protocol-level issues (XML responses, headers, error codes).

**When to add**: Once basic S3 operations are stable (after Tier 1 and Tier 2 pass).

**Setup notes**: Requires Python environment. Cache the tox virtualenv in CI. Filter out tests for unimplemented operations using pytest markers.

#### Tier 4: MSST-S3 (Nice to Have -- Later)

**Approach**: Use MSST-S3's LocalStack Docker Compose profile.

**Rationale**: Largest test count and multi-backend comparison. However, there is significant overlap with Ceph s3-tests. Consider adopting if we need a single comprehensive suite to replace Ceph s3-tests, or for periodic compliance benchmarking.

### What NOT to Adopt

- **Custom pytest+boto3 tests from scratch**: The Ceph s3-tests already cover this extensively. Writing our own would duplicate effort.
- **pytest-localstack plugin**: Designed for testing applications that use LocalStack, not for testing LocalStack itself.
- **SNIA Plugfest tools**: Not yet publicly available as standalone test suites.

### CI Pipeline Design

```
PR Checks (fast feedback):
  - cargo build
  - cargo test (Tier 1 Rust tests)
  - cargo clippy, cargo fmt

Merge / Nightly (thorough validation):
  - Start localstack-rs server
  - Run Mint (Tier 2)
  - Run Ceph s3-tests (Tier 3)
  - Upload test artifacts (logs, JUnit XML)
```

### Implementation Priority

1. **Immediate**: Rust integration tests in `tests/` using `aws-sdk-s3`
2. **Week 2-3**: Mint Docker test in CI (add `scripts/e2e-mint.sh`)
3. **Month 2**: Ceph s3-tests integration (add `scripts/e2e-s3tests.sh`)
4. **Later**: MSST-S3 for comprehensive benchmarking

---

## Sources

- [Ceph s3-tests](https://github.com/ceph/s3-tests) - S3 compatibility tests for S3 clones
- [MinIO Mint](https://github.com/minio/mint) - Collection of tests to detect overall correctness of MinIO server
- [MSST-S3](https://github.com/linux-kdevops/msst-s3) - An S3 interoperability test suite
- [s3s Project](https://github.com/Nugine/s3s) - S3 Service Adapter (Rust)
- [s3s-e2e crate](https://github.com/Nugine/s3s/tree/main/crates/s3s-e2e) - s3s end-to-end tests
- [s3s-test crate](https://github.com/Nugine/s3s/tree/main/crates/s3s-test) - s3s test framework
- [s3s CI configuration](https://github.com/Nugine/s3s/blob/main/.github/workflows/ci.yml) - s3s GitHub Actions CI pipeline
- [pytest-localstack](https://github.com/mintel/pytest-localstack) - Pytest plugin for local AWS integration tests
- [LocalStack GitHub Actions](https://blog.localstack.cloud/automate-your-tests-with-github-actions-and-localstack/) - Automate tests with GitHub Actions and LocalStack
- [SNIA Cloud Object Storage Plugfest](https://www.snia.org/node/13718) - Industry S3 compatibility testing initiative
- [S3 API Testing Framework (DeepWiki)](https://deepwiki.com/ceph/s3-tests/3-s3-api-testing-framework) - Ceph s3-tests framework analysis
- [Testcontainers + LocalStack](https://testcontainers.com/guides/testing-aws-service-integrations-using-localstack/) - Testing AWS service integrations
