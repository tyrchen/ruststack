# RustStack S3: Smithy-Based Redesign (Replacing s3s)

**Date:** 2026-02-27
**Status:** Draft / RFC
**Depends on:** [rust-rewrite-feasibility.md](./rust-rewrite-feasibility.md), [smithy-rs-server-codegen-research](../docs/research/smithy-rs-server-codegen-research.md)
**Scope:** Replace the s3s dependency with a smithy-rs-generated server SDK and custom HTTP layer, giving full ownership of the S3 protocol stack.

---

## Table of Contents

1. [Motivation](#1-motivation)
2. [Goals and Non-Goals](#2-goals-and-non-goals)
3. [Architecture Overview](#3-architecture-overview)
4. [Smithy Code Generation Strategy](#4-smithy-code-generation-strategy)
5. [HTTP Layer Design](#5-http-layer-design)
6. [XML Serialization Layer](#6-xml-serialization-layer)
7. [SigV4 Authentication](#7-sigv4-authentication)
8. [Virtual Hosting & Request Routing](#8-virtual-hosting--request-routing)
9. [Error Handling & S3 Error Formatting](#9-error-handling--s3-error-formatting)
10. [Crate Structure (Revised)](#10-crate-structure-revised)
11. [What We Keep From Current Implementation](#11-what-we-keep-from-current-implementation)
12. [What We Build New](#12-what-we-build-new)
13. [Migration Plan](#13-migration-plan)
14. [Risk Analysis](#14-risk-analysis)
15. [Open Questions](#15-open-questions)

---

## 1. Motivation

### 1.1 Why Remove s3s?

The current implementation builds on the `s3s` crate (v0.11), which provides:
- 96-operation S3 trait with DTOs
- HTTP routing via hyper
- XML serialization/deserialization
- SigV4 authentication
- Virtual-hosted-style addressing

While s3s accelerated our initial development, several issues have emerged:

| Issue | Impact |
|-------|--------|
| **Dependency instability**: s3s v0.12 shipped with broken RC crypto deps (sha2 0.11.0-rc.3, hmac 0.13.0-rc.3) that break compilation | Build reliability |
| **Auth model mismatch**: s3s's `default_check` in access layer requires `cx.credentials()` to be `Some`, causing 403 errors when auth is configured but validation is skipped. Debugging this required reading s3s internals | Operational correctness |
| **Opaque HTTP layer**: s3s owns the HTTP server, routing, and middleware stack. Adding custom middleware (health checks, CORS preflight, metrics) requires wrapping the s3s service externally | Extensibility |
| **Limited error control**: s3s formats S3 errors internally. We cannot customize error responses, add request IDs, or match LocalStack's exact error formatting | API compatibility |
| **Single-maintainer risk**: s3s is maintained by one person (Nugine). Bus factor = 1 | Long-term viability |
| **Smithy model lag**: s3s generates its own types from a bundled Smithy model. We cannot easily update to the latest S3 API model or add custom operations | API coverage |

### 1.2 Why Smithy Server Codegen?

AWS's Smithy models are the **authoritative** source for S3 API definitions. Using smithy-rs to generate server types from these models means:

1. **Type correctness**: Generated Input/Output/Error types exactly match what AWS SDKs expect
2. **Model updates**: When AWS adds new S3 operations, we regenerate from the updated model
3. **Protocol fidelity**: The RestXml serialization follows the same spec as the real AWS S3
4. **No middleman**: We depend on the official AWS model, not a third-party interpretation

### 1.3 What This Spec Proposes

A **hybrid approach**:
- **Use smithy-rs codegen** to generate S3 operation types (Input/Output/Error structs) and optionally routing logic from the official AWS S3 Smithy model
- **Build our own HTTP layer** using hyper + tower for full control over the request/response pipeline
- **Build our own XML serialization** using `quick-xml`, guided by the Smithy model's HTTP binding traits
- **Build our own SigV4 auth** for request signature verification
- **Build our own virtual hosting** for bucket name extraction from Host headers
- **Keep all business logic** from the current `ruststack-s3-core` (state, storage, checksums, cors, etc.)

---

## 2. Goals and Non-Goals

### Goals

1. **Remove s3s dependency entirely** -- no `s3s`, `s3s-aws`, or `s3s-*` crates in the dependency tree
2. **Full ownership of the HTTP pipeline** -- we control routing, middleware, auth, error formatting, and response serialization
3. **Smithy model as the single source of truth** -- all S3 operation types generated from the official AWS S3 Smithy JSON model
4. **Preserve all existing business logic** -- state management, storage backend, checksums, CORS, validation, and all 60+ operation handlers continue to work
5. **Pass existing integration tests** -- all 45 integration tests and 242 unit tests continue to pass
6. **Improve debuggability** -- when something goes wrong in the HTTP/auth/routing layer, we can debug and fix it without reading third-party internals

### Non-Goals

1. **Use smithy-rs Gradle/JDK pipeline in CI** -- we pre-generate types and commit them, avoiding JDK dependency in the build
2. **Generate the full smithy-rs server SDK** -- we only generate types, not the full server framework (routing, serialization, builder)
3. **Support protocols other than RestXml** -- S3 only uses RestXml; we don't need generic protocol support
4. **Rewrite business logic** -- the ops/ module handlers stay structurally the same

---

## 3. Architecture Overview

### 3.1 Current Architecture (s3s-based)

```
AWS SDK / CLI
     │ HTTP :4566
     ▼
┌─────────────────────────────────────────────┐
│  s3s HTTP Layer (OPAQUE)                    │
│  - hyper server (s3s controls)              │
│  - S3 request routing (s3s controls)        │
│  - XML deserialization (s3s controls)       │
│  - SigV4 auth (s3s controls)               │
│  - Virtual hosting (s3s controls)           │
│  - Error formatting (s3s controls)          │
└──────────────┬──────────────────────────────┘
               │ S3Request<Input>
               ▼
┌─────────────────────────────────────────────┐
│  RustStackS3 (impl s3s::S3)                │
│  - 60+ operation handlers                   │
│  - State management (DashMap)               │
│  - Storage backend (InMemory)               │
│  - Checksums, CORS, validation              │
└─────────────────────────────────────────────┘
```

### 3.2 New Architecture (Smithy-based, self-owned)

```
AWS SDK / CLI
     │ HTTP :4566
     ▼
┌─────────────────────────────────────────────┐
│  ruststack-s3-http (WE OWN)                │
│  ┌────────────────────────────────────────┐ │
│  │ hyper Server + tower Middleware        │ │
│  │ ┌──────┐ ┌──────┐ ┌──────┐ ┌───────┐ │ │
│  │ │Trace │→│ CORS │→│Health│→│Metrics│ │ │
│  │ └──────┘ └──────┘ └──────┘ └───────┘ │ │
│  └────────────────────┬───────────────────┘ │
│  ┌────────────────────┴───────────────────┐ │
│  │ S3 Router (WE OWN)                    │ │
│  │ - Virtual hosting (Host → bucket)     │ │
│  │ - Path-style fallback                 │ │
│  │ - Method + path + query → Operation   │ │
│  │ - SigV4 verification (optional)       │ │
│  └────────────────────┬───────────────────┘ │
│  ┌────────────────────┴───────────────────┐ │
│  │ XML Deserializer (WE OWN)             │ │
│  │ - HTTP headers/query → struct fields  │ │
│  │ - XML body → struct fields            │ │
│  │ - Streaming body passthrough          │ │
│  └────────────────────┬───────────────────┘ │
└────────────────────────┼────────────────────┘
                         │ S3Operation<Input>
                         ▼
┌─────────────────────────────────────────────┐
│  ruststack-s3-core (MOSTLY UNCHANGED)       │
│  ┌────────────────────────────────────────┐ │
│  │ Operation Dispatch                    │ │
│  │ - Typed handler per operation         │ │
│  │ - Input types from smithy codegen     │ │
│  └────────────────────┬───────────────────┘ │
│  ┌────────────────────┴───────────────────┐ │
│  │ State + Storage (UNCHANGED)           │ │
│  │ - S3ServiceState, S3Bucket, KeyStore  │ │
│  │ - InMemoryStorage with spillover      │ │
│  │ - Checksums, CORS, validation         │ │
│  └────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────┐
│  XML Serializer (WE OWN)                   │
│  - Struct fields → XML body               │
│  - Struct fields → HTTP headers            │
│  - S3-specific error XML formatting        │
│  - Streaming body passthrough              │
└─────────────────────────────────────────────┘
```

### 3.3 Key Architectural Principles

1. **Generated types, hand-built plumbing**: Smithy codegen gives us correct types; we build everything else
2. **Layered middleware**: Each concern (routing, auth, CORS, health) is a separate tower `Layer`
3. **No trait object dispatch for operations**: Use an enum-based dispatch or match on operation name for zero-overhead routing
4. **Streaming first**: Body handling uses `hyper::body::Incoming` directly, no intermediate buffering for large objects

---

## 4. Smithy Code Generation Strategy

### 4.1 Approach: Pre-Generated Types, Committed to Repo

We use smithy-rs codegen as a **developer-time** tool, not a build-time dependency:

```
Developer machine (one-time or on model update):
  ┌──────────────────┐     ┌──────────────────┐
  │ AWS S3 Smithy    │ ──→ │ smithy-rs codegen │ ──→ Generated Rust types
  │ model (JSON)     │     │ (Gradle + JDK)   │     (committed to repo)
  └──────────────────┘     └──────────────────┘

CI / Developer build:
  ┌──────────────────┐
  │ cargo build      │  ← Pure Rust, no JDK needed
  │ (uses committed  │
  │  generated types)│
  └──────────────────┘
```

**Why commit generated code?**
- No JDK/Gradle in CI or developer setup
- Deterministic builds
- Code review on type changes when the S3 model updates
- Same approach aws-sdk-rust uses (types are published as crates, not generated at build time)

### 4.2 What We Generate

From the S3 Smithy model, we generate **only the types**:

| Generated | Purpose | Example |
|-----------|---------|---------|
| Operation input structs | Request data | `PutObjectInput { bucket, key, body, ... }` |
| Operation output structs | Response data | `PutObjectOutput { etag, version_id, ... }` |
| Operation error enums | Error variants | `PutObjectError::NoSuchBucket(...)` |
| Shape types (enums, structs) | Shared types | `BucketLocationConstraint`, `ServerSideEncryption`, `StorageClass` |
| Primitive wrappers | Smithy constrained types | `BucketName(String)`, `ObjectKey(String)` |

We **do NOT** generate:
- HTTP routing logic (we build our own, more flexible router)
- XML serialization/deserialization (we build our own with `quick-xml`)
- Service builder / server framework (we use hyper + tower directly)
- Auth middleware (we build our own SigV4)

### 4.3 Alternative: Custom Smithy Model Parser

If the smithy-rs codegen proves too cumbersome for just generating types, we can build a lightweight Rust tool that:

1. Reads the S3 Smithy JSON AST (available at `aws/api-models-aws` on GitHub)
2. Extracts operation shapes, input/output/error shapes, and member types
3. Generates Rust structs with `serde` derives and XML serialization attributes
4. Outputs a single `s3_types.rs` file (or module tree)

This avoids the Gradle/JDK dependency entirely. The Smithy JSON AST is well-documented and relatively simple to parse.

```rust
// Example generated type from Smithy model:
/// Input for the PutObject operation.
#[derive(Debug, Clone, Default)]
pub struct PutObjectInput {
    /// The bucket name.
    pub bucket: String,
    /// Object key.
    pub key: String,
    /// Object data (streaming).
    pub body: Option<StreamingBlob>,
    /// A standard MIME type describing the format of the object data.
    pub content_type: Option<String>,
    /// Size of the body in bytes.
    pub content_length: Option<i64>,
    /// The base64-encoded 128-bit MD5 digest of the message.
    pub content_md5: Option<String>,
    /// The account ID of the expected bucket owner.
    pub expected_bucket_owner: Option<String>,
    /// The server-side encryption algorithm used.
    pub server_side_encryption: Option<ServerSideEncryption>,
    /// Storage class for the object.
    pub storage_class: Option<StorageClass>,
    /// Tagging header (URL-encoded key=value pairs).
    pub tagging: Option<String>,
    /// Object metadata (x-amz-meta-* headers).
    pub metadata: Option<HashMap<String, String>>,
    // ... 30+ more fields from the Smithy model
}
```

### 4.4 Codegen Tooling Setup

```
codegen/
├── README.md                    # How to regenerate types
├── Makefile                     # make generate-s3-types
├── smithy-model/
│   └── s3.json                  # AWS S3 Smithy JSON AST (pinned version)
├── generator/
│   ├── Cargo.toml               # Rust binary that reads Smithy JSON → Rust code
│   └── src/
│       ├── main.rs              # Entry point
│       ├── model.rs             # Smithy JSON AST parser
│       ├── shapes.rs            # Shape type resolution
│       └── codegen.rs           # Rust code emitter
└── output/                      # Generated code (review before committing)
```

Alternatively, if using smithy-rs:

```
codegen/
├── README.md                    # How to regenerate types
├── Makefile                     # make generate-s3-types (requires JDK 17+)
├── build.gradle.kts             # Smithy codegen configuration
├── settings.gradle.kts          # Gradle settings
├── gradle/                      # Gradle wrapper (committed)
├── smithy-model/
│   └── s3.json                  # AWS S3 Smithy JSON AST
└── smithy-build.json            # Codegen plugin configuration
```

**Recommendation**: Build the custom Smithy JSON parser in Rust. It's less than 2,000 LOC, eliminates the JVM dependency entirely, and gives us precise control over what gets generated and how. The S3 Smithy model has well-defined structure.

---

## 5. HTTP Layer Design

### 5.1 Server Setup

Replace s3s's HTTP handling with direct hyper + tower:

```rust
use hyper::body::Incoming;
use hyper::service::Service;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as HttpConnBuilder;
use tower::ServiceBuilder;

/// The top-level HTTP service that handles all incoming requests.
#[derive(Clone)]
pub struct S3HttpService {
    router: S3Router,
    provider: Arc<RustStackS3>,
    config: Arc<S3Config>,
}

impl Service<Request<Incoming>> for S3HttpService {
    type Response = Response<S3ResponseBody>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let this = self.clone();
        Box::pin(async move {
            Ok(this.handle_request(req).await)
        })
    }
}

impl S3HttpService {
    async fn handle_request(&self, req: Request<Incoming>) -> Response<S3ResponseBody> {
        // 1. Health check interception
        if self.is_health_check(&req) {
            return self.health_response();
        }

        // 2. CORS preflight
        if req.method() == Method::OPTIONS {
            return self.handle_cors_preflight(&req).await;
        }

        // 3. Extract bucket from Host/path (virtual hosting)
        let routing_ctx = match self.router.resolve(&req) {
            Ok(ctx) => ctx,
            Err(e) => return self.error_response(e),
        };

        // 4. SigV4 auth (optional)
        if !self.config.s3_skip_signature_validation {
            if let Err(e) = self.verify_signature(&req, &routing_ctx).await {
                return self.error_response(e);
            }
        }

        // 5. Route to operation handler
        let result = self.dispatch(routing_ctx, req).await;

        // 6. Serialize response (XML + headers)
        match result {
            Ok(response) => self.add_cors_headers(response),
            Err(e) => self.error_response(e),
        }
    }
}
```

### 5.2 Tower Middleware Stack

```rust
let service = ServiceBuilder::new()
    // Outermost: request tracing
    .layer(TraceLayer::new_for_http())
    // Request ID generation
    .layer(RequestIdLayer::new())
    // Optional: rate limiting
    .layer(RateLimitLayer::new(/* config */))
    // The S3 service
    .service(S3HttpService::new(provider, config));
```

### 5.3 Response Body Type

```rust
/// S3 response body that supports both buffered and streaming modes.
pub enum S3ResponseBody {
    /// Small responses: XML, errors, empty bodies.
    Buffered(Bytes),
    /// Large responses: streaming object data.
    Stream(Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send>>),
    /// Empty body (204 responses, HEAD responses).
    Empty,
}

impl hyper::body::Body for S3ResponseBody {
    type Data = Bytes;
    type Error = io::Error;
    // ... Frame-based implementation
}
```

---

## 6. XML Serialization Layer

### 6.1 Design

S3 uses the RestXml protocol with `noErrorWrapping: true`. Key characteristics:
- Request bodies are XML for operations like `PutBucketVersioning`, `CompleteMultipartUpload`
- Response bodies are XML for list operations, errors, and some metadata responses
- Most operation data is in HTTP headers and query parameters, not XML
- S3 has quirks: `@s3UnwrappedXmlOutput`, no error wrapping, custom namespace handling

### 6.2 Deserialization (Request)

```rust
/// Trait for deserializing S3 request data from HTTP requests.
pub trait FromS3Request: Sized {
    /// Extract operation input from the HTTP request.
    ///
    /// Reads from:
    /// - URI path parameters (bucket, key)
    /// - Query string parameters
    /// - HTTP headers (x-amz-*, content-type, etc.)
    /// - XML body (if present)
    /// - Streaming body (for PutObject, UploadPart)
    fn from_s3_request(
        req: &Parts,
        bucket: &str,
        key: Option<&str>,
        body: RequestBody,
    ) -> Result<Self, S3Error>;
}

// Example implementation for PutBucketVersioningInput:
impl FromS3Request for PutBucketVersioningInput {
    fn from_s3_request(
        req: &Parts,
        bucket: &str,
        _key: Option<&str>,
        body: RequestBody,
    ) -> Result<Self, S3Error> {
        let xml_body = body.collect_bytes().await?;
        let config: VersioningConfiguration = deserialize_xml(&xml_body)?;

        Ok(Self {
            bucket: bucket.to_owned(),
            versioning_configuration: config,
            expected_bucket_owner: header_opt(req, "x-amz-expected-bucket-owner"),
            mfa: header_opt(req, "x-amz-mfa"),
            ..Default::default()
        })
    }
}
```

### 6.3 Serialization (Response)

```rust
/// Trait for serializing S3 response data into HTTP responses.
pub trait IntoS3Response {
    /// Convert operation output to an HTTP response.
    ///
    /// Writes to:
    /// - HTTP status code
    /// - Response headers
    /// - XML body (for structured responses)
    /// - Streaming body (for GetObject)
    fn into_s3_response(self) -> Result<Response<S3ResponseBody>, S3Error>;
}

// Example implementation for ListObjectsV2Output:
impl IntoS3Response for ListObjectsV2Output {
    fn into_s3_response(self) -> Result<Response<S3ResponseBody>, S3Error> {
        let xml = serialize_xml("ListBucketResult", &self)?;

        Ok(Response::builder()
            .status(200)
            .header("content-type", "application/xml")
            .body(S3ResponseBody::Buffered(xml.into()))?)
    }
}
```

### 6.4 XML Utilities

```rust
/// Serialize a struct to S3-compatible XML.
///
/// Uses `quick-xml` with the S3 namespace (`http://s3.amazonaws.com/doc/2006-03-01/`).
pub fn serialize_xml<T: S3Serialize>(root_element: &str, value: &T) -> Result<String, S3Error> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let mut root = BytesStart::new(root_element);
    root.push_attribute(("xmlns", "http://s3.amazonaws.com/doc/2006-03-01/"));
    writer.write_event(Event::Start(root))?;

    value.serialize_xml(&mut writer)?;

    writer.write_event(Event::End(BytesEnd::new(root_element)))?;
    Ok(String::from_utf8(writer.into_inner().into_inner())?)
}

/// Deserialize S3-compatible XML into a struct.
pub fn deserialize_xml<T: S3Deserialize>(xml: &[u8]) -> Result<T, S3Error> {
    let reader = Reader::from_reader(xml);
    T::deserialize_xml(reader)
}

/// Derive-like trait for S3 XML serialization.
/// Will be implemented by hand for each type, or generated by our codegen tool.
pub trait S3Serialize {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), S3Error>;
}

pub trait S3Deserialize: Sized {
    fn deserialize_xml<R: BufRead>(reader: Reader<R>) -> Result<Self, S3Error>;
}
```

### 6.5 Codegen for Serialization

The Smithy model defines HTTP bindings for each operation:
- `@httpHeader("x-amz-request-id")` → extract from / write to HTTP header
- `@httpQuery("delimiter")` → extract from / write to query parameter
- `@httpLabel` → extract from URI path
- `@httpPayload` → the XML or streaming body
- `@xmlName("Contents")` → XML element name override
- `@xmlFlattened` → list serialization without wrapper element

Our codegen tool generates `FromS3Request` and `IntoS3Response` implementations by reading these Smithy traits from the model.

---

## 7. SigV4 Authentication

### 7.1 Design

Build a standalone SigV4 verifier that:
1. Parses the `Authorization` header or query parameters (presigned URLs)
2. Reconstructs the canonical request
3. Derives the signing key from the secret key
4. Compares HMAC-SHA256 signatures

```rust
/// SigV4 authentication result.
pub struct AuthResult {
    pub access_key_id: String,
    pub region: String,
    pub service: String,
    pub signed_headers: Vec<String>,
}

/// Verify SigV4 signature on an incoming request.
pub fn verify_sigv4(
    req: &Parts,
    body_hash: &str,
    credential_provider: &dyn CredentialProvider,
) -> Result<AuthResult, S3Error> {
    // 1. Parse Authorization header
    let auth = parse_auth_header(req)?;

    // 2. Build canonical request
    let canonical = build_canonical_request(req, &auth.signed_headers, body_hash)?;

    // 3. Build string to sign
    let string_to_sign = build_string_to_sign(&auth, &canonical)?;

    // 4. Look up secret key
    let secret_key = credential_provider.get_secret_key(&auth.access_key_id)?;

    // 5. Derive signing key
    let signing_key = derive_signing_key(&secret_key, &auth.date, &auth.region, &auth.service);

    // 6. Calculate expected signature
    let expected = hmac_sha256(&signing_key, string_to_sign.as_bytes());

    // 7. Constant-time comparison
    if !constant_time_eq(expected.as_ref(), &hex::decode(&auth.signature)?) {
        return Err(S3Error::signature_does_not_match());
    }

    Ok(AuthResult {
        access_key_id: auth.access_key_id,
        region: auth.region,
        service: auth.service,
        signed_headers: auth.signed_headers,
    })
}

/// Verify presigned URL (SigV4 in query parameters).
pub fn verify_presigned_url(
    req: &Parts,
    credential_provider: &dyn CredentialProvider,
) -> Result<AuthResult, S3Error> {
    // X-Amz-Algorithm, X-Amz-Credential, X-Amz-Date,
    // X-Amz-Expires, X-Amz-SignedHeaders, X-Amz-Signature
    // ...
}

/// Credential lookup trait.
pub trait CredentialProvider: Send + Sync {
    fn get_secret_key(&self, access_key_id: &str) -> Result<String, S3Error>;
}
```

### 7.2 Dev Mode (Skip Validation)

When `S3_SKIP_SIGNATURE_VALIDATION=true`:
- Skip signature verification entirely
- Still parse the Authorization header to extract access key and region (for multi-account)
- Accept any credentials

```rust
impl S3HttpService {
    async fn verify_signature(
        &self,
        req: &Request<Incoming>,
        ctx: &RoutingContext,
    ) -> Result<(), S3Error> {
        if self.config.s3_skip_signature_validation {
            return Ok(()); // No auth at all -- no dummy keys, no signature dance
        }
        verify_sigv4(req.parts(), "UNSIGNED-PAYLOAD", &self.credentials)?;
        Ok(())
    }
}
```

This eliminates the s3s auth mismatch bug entirely. When we skip validation, we truly skip it -- no `default_check`, no credential lookup, no signature comparison.

---

## 8. Virtual Hosting & Request Routing

### 8.1 Bucket Extraction

```rust
pub struct S3Router {
    /// Domain for virtual-hosted-style requests (e.g., "s3.localhost.localstack.cloud")
    domain: String,
    /// Whether virtual hosting is enabled
    virtual_hosting: bool,
}

pub struct RoutingContext {
    pub bucket: Option<String>,
    pub key: Option<String>,
    pub operation: S3Operation,
    pub query_params: HashMap<String, String>,
}

impl S3Router {
    /// Resolve an incoming HTTP request to an S3 operation.
    pub fn resolve(&self, req: &Request<Incoming>) -> Result<RoutingContext, S3Error> {
        let (bucket, remaining_path) = self.extract_bucket(req)?;
        let key = Self::extract_key(&remaining_path);
        let query_params = Self::parse_query(req.uri().query());
        let operation = Self::identify_operation(req.method(), &bucket, &key, &query_params)?;

        Ok(RoutingContext { bucket, key, operation, query_params })
    }

    /// Extract bucket name from Host header (virtual-hosted) or path (path-style).
    fn extract_bucket(&self, req: &Request<Incoming>) -> Result<(Option<String>, String), S3Error> {
        if self.virtual_hosting {
            if let Some(host) = req.headers().get("host").and_then(|h| h.to_str().ok()) {
                // Strip port if present
                let host = host.split(':').next().unwrap_or(host);

                // Check if host is a subdomain of our S3 domain
                if let Some(bucket) = host.strip_suffix(&format!(".{}", self.domain)) {
                    if !bucket.is_empty() {
                        return Ok((Some(bucket.to_owned()), req.uri().path().to_owned()));
                    }
                }
            }
        }

        // Path-style: /bucket/key or /bucket
        let path = req.uri().path();
        let trimmed = path.trim_start_matches('/');
        if trimmed.is_empty() {
            return Ok((None, String::new())); // ListBuckets
        }

        let (bucket, rest) = match trimmed.split_once('/') {
            Some((b, r)) => (b.to_owned(), format!("/{r}")),
            None => (trimmed.to_owned(), String::new()),
        };

        Ok((Some(bucket), rest))
    }
}
```

### 8.2 Operation Identification

S3 operations are identified by HTTP method + path structure + query parameters:

```rust
/// All S3 operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum S3Operation {
    // Bucket CRUD
    CreateBucket,
    DeleteBucket,
    HeadBucket,
    ListBuckets,
    GetBucketLocation,

    // Bucket config (sub-resources)
    GetBucketVersioning,
    PutBucketVersioning,
    GetBucketEncryption,
    PutBucketEncryption,
    DeleteBucketEncryption,
    // ... 40+ more bucket config operations

    // Object CRUD
    PutObject,
    GetObject,
    HeadObject,
    DeleteObject,
    DeleteObjects,
    CopyObject,
    // ... more object operations

    // Multipart
    CreateMultipartUpload,
    UploadPart,
    UploadPartCopy,
    CompleteMultipartUpload,
    AbortMultipartUpload,
    ListParts,
    ListMultipartUploads,

    // List
    ListObjects,
    ListObjectsV2,
    ListObjectVersions,
}

impl S3Router {
    fn identify_operation(
        method: &Method,
        bucket: &Option<String>,
        key: &Option<String>,
        query: &HashMap<String, String>,
    ) -> Result<S3Operation, S3Error> {
        // Sub-resource query parameters determine the operation
        // e.g., ?versioning, ?encryption, ?cors, ?tagging, ?uploads, etc.

        match (method, bucket.is_some(), key.is_some()) {
            // No bucket: ListBuckets
            (&Method::GET, false, false) => Ok(S3Operation::ListBuckets),

            // Bucket-level operations (no key)
            (&Method::PUT, true, false) if query.is_empty() => Ok(S3Operation::CreateBucket),
            (&Method::DELETE, true, false) if query.is_empty() => Ok(S3Operation::DeleteBucket),
            (&Method::HEAD, true, false) => Ok(S3Operation::HeadBucket),

            // Sub-resource operations (bucket + query param)
            (&Method::GET, true, false) if query.contains_key("versioning") =>
                Ok(S3Operation::GetBucketVersioning),
            (&Method::PUT, true, false) if query.contains_key("versioning") =>
                Ok(S3Operation::PutBucketVersioning),

            // ... pattern matching for all operations based on method + query params

            // Object operations (bucket + key)
            (&Method::PUT, true, true) if !is_copy_request(/* headers */) =>
                Ok(S3Operation::PutObject),
            (&Method::PUT, true, true) /* if copy source header present */ =>
                Ok(S3Operation::CopyObject),
            (&Method::GET, true, true) => Ok(S3Operation::GetObject),
            (&Method::HEAD, true, true) => Ok(S3Operation::HeadObject),
            (&Method::DELETE, true, true) => Ok(S3Operation::DeleteObject),

            // List operations
            (&Method::GET, true, false) if query.contains_key("list-type") =>
                Ok(S3Operation::ListObjectsV2),
            (&Method::GET, true, false) => Ok(S3Operation::ListObjects),

            _ => Err(S3Error::not_implemented("Unknown operation")),
        }
    }
}
```

---

## 9. Error Handling & S3 Error Formatting

### 9.1 S3 Error Type

```rust
/// S3 error with full AWS-compatible error response formatting.
#[derive(Debug, thiserror::Error)]
pub struct S3Error {
    pub code: S3ErrorCode,
    pub message: String,
    pub resource: Option<String>,
    pub request_id: Option<String>,
    pub status_code: StatusCode,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

/// S3-specific error codes.
#[derive(Debug, Clone, Copy)]
pub enum S3ErrorCode {
    AccessDenied,
    BucketAlreadyExists,
    BucketAlreadyOwnedByYou,
    BucketNotEmpty,
    EntityTooLarge,
    EntityTooSmall,
    InternalError,
    InvalidArgument,
    InvalidBucketName,
    InvalidPart,
    InvalidPartOrder,
    InvalidRange,
    InvalidRequest,
    MalformedXML,
    MethodNotAllowed,
    NoSuchBucket,
    NoSuchKey,
    NoSuchUpload,
    NoSuchVersion,
    NotImplemented,
    PreconditionFailed,
    SignatureDoesNotMatch,
    // ... complete list
}
```

### 9.2 XML Error Response

```rust
impl S3Error {
    /// Format as S3-standard XML error response.
    ///
    /// S3 uses `noErrorWrapping: true`, so the error XML is:
    /// ```xml
    /// <?xml version="1.0" encoding="UTF-8"?>
    /// <Error>
    ///   <Code>NoSuchBucket</Code>
    ///   <Message>The specified bucket does not exist</Message>
    ///   <BucketName>my-bucket</BucketName>
    ///   <RequestId>tx00000000000000000001-00...</RequestId>
    /// </Error>
    /// ```
    pub fn to_xml_response(&self) -> Response<S3ResponseBody> {
        let xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <Error>\n\
               <Code>{code}</Code>\n\
               <Message>{message}</Message>\n\
               {resource}\
               <RequestId>{request_id}</RequestId>\n\
             </Error>",
            code = self.code.as_str(),
            message = xml_escape(&self.message),
            resource = self.resource.as_ref().map_or(String::new(), |r|
                format!("  <Resource>{}</Resource>\n", xml_escape(r))),
            request_id = self.request_id.as_deref().unwrap_or("0"),
        );

        Response::builder()
            .status(self.status_code)
            .header("content-type", "application/xml")
            .header("x-amz-request-id", self.request_id.as_deref().unwrap_or("0"))
            .body(S3ResponseBody::Buffered(xml.into()))
            .unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(S3ResponseBody::Empty)
                    .expect("empty response should always build")
            })
    }
}
```

---

## 10. Crate Structure (Revised)

```
localstack-rs/
├── Cargo.toml                       # Workspace root
├── Cargo.lock
├── rust-toolchain.toml
│
├── crates/
│   ├── ruststack-core/              # UNCHANGED - Core types, AccountRegionStore
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs
│   │       ├── error.rs
│   │       ├── state.rs
│   │       └── types.rs
│   │
│   ├── ruststack-s3-model/          # NEW - Generated S3 types from Smithy model
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs               # Re-exports
│   │       ├── operations.rs        # S3Operation enum
│   │       ├── types.rs             # Shared Smithy shapes (enums, newtypes)
│   │       ├── input/               # Per-operation input structs
│   │       │   ├── mod.rs
│   │       │   ├── bucket.rs        # CreateBucketInput, DeleteBucketInput, ...
│   │       │   ├── object.rs        # PutObjectInput, GetObjectInput, ...
│   │       │   ├── multipart.rs     # CreateMultipartUploadInput, ...
│   │       │   └── config.rs        # PutBucketVersioningInput, ...
│   │       ├── output/              # Per-operation output structs
│   │       │   ├── mod.rs
│   │       │   ├── bucket.rs
│   │       │   ├── object.rs
│   │       │   ├── multipart.rs
│   │       │   └── config.rs
│   │       └── error.rs             # S3ErrorCode, S3Error
│   │
│   ├── ruststack-s3-xml/            # NEW - S3 XML serialization/deserialization
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── serialize.rs         # Struct → XML
│   │       ├── deserialize.rs       # XML → Struct
│   │       └── traits.rs            # S3Serialize, S3Deserialize traits
│   │
│   ├── ruststack-s3-auth/           # NEW - SigV4 authentication
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── sigv4.rs             # SigV4 signature verification
│   │       ├── presigned.rs         # Presigned URL verification
│   │       ├── canonical.rs         # Canonical request construction
│   │       └── credentials.rs       # Credential provider trait
│   │
│   ├── ruststack-s3-http/           # NEW - S3 HTTP layer (routing, middleware)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── service.rs           # S3HttpService (hyper Service impl)
│   │       ├── router.rs            # S3Router (virtual hosting + operation ID)
│   │       ├── dispatch.rs          # Route operation → handler
│   │       ├── request.rs           # FromS3Request implementations
│   │       ├── response.rs          # IntoS3Response implementations
│   │       ├── body.rs              # S3ResponseBody type
│   │       └── middleware/
│   │           ├── mod.rs
│   │           ├── cors.rs          # CORS preflight + response headers
│   │           ├── trace.rs         # Request tracing
│   │           └── request_id.rs    # x-amz-request-id generation
│   │
│   ├── ruststack-s3-core/           # REFACTORED - Business logic (no s3s dependency)
│   │   ├── Cargo.toml               # Remove s3s, s3s-aws; add ruststack-s3-model
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── provider.rs          # RustStackS3 (same struct, new trait impls)
│   │       ├── ops/                 # REFACTORED - Use our types instead of s3s types
│   │       │   ├── mod.rs           # No longer `impl s3s::S3`; uses our dispatch
│   │       │   ├── bucket.rs        # handle_create_bucket(CreateBucketInput) → ...
│   │       │   ├── bucket_config.rs
│   │       │   ├── object.rs
│   │       │   ├── object_config.rs
│   │       │   ├── multipart.rs
│   │       │   └── list.rs
│   │       ├── state/               # UNCHANGED
│   │       ├── storage.rs           # UNCHANGED
│   │       ├── checksums.rs         # UNCHANGED
│   │       ├── cors.rs              # UNCHANGED
│   │       ├── validation.rs        # UNCHANGED
│   │       ├── utils.rs             # UNCHANGED
│   │       └── error.rs             # REFACTORED - Use S3Error from s3-model
│   │
│   └── ruststack-core/              # UNCHANGED
│
├── apps/
│   └── ruststack-s3-server/         # REFACTORED - Use new HTTP layer
│       ├── Cargo.toml
│       └── src/
│           └── main.rs              # Uses S3HttpService instead of s3s
│
├── codegen/                          # NEW - Smithy model → Rust type generator
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   └── ...
│   └── smithy-model/
│       └── s3.json                   # Pinned AWS S3 Smithy model
│
└── tests/
    └── integration/                  # UNCHANGED - Same tests, still pass
```

### 10.1 Dependency Graph

```
ruststack-s3-server (binary)
├── ruststack-s3-http
│   ├── ruststack-s3-core
│   │   ├── ruststack-s3-model      (generated types)
│   │   ├── ruststack-core           (AccountRegionStore, etc.)
│   │   └── (crypto, dashmap, etc.)
│   ├── ruststack-s3-model
│   ├── ruststack-s3-xml
│   │   └── ruststack-s3-model
│   ├── ruststack-s3-auth
│   │   └── (hmac, sha2, etc.)
│   ├── hyper, hyper-util, tower
│   └── tracing
└── tokio
```

**Key change**: No `s3s` or `s3s-aws` anywhere in the tree.

---

## 11. What We Keep From Current Implementation

| Component | Location | Changes Needed |
|-----------|----------|---------------|
| State management (S3ServiceState, S3Bucket, KeyStore, VersionedKeyStore) | `s3-core/src/state/` | **None** - internal types, not coupled to s3s |
| Storage backend (InMemoryStorage) | `s3-core/src/storage.rs` | **None** - pure Bytes/tempfile handling |
| Checksum computation (CRC32, CRC32C, SHA1, SHA256, MD5) | `s3-core/src/checksums.rs` | **None** - pure hash functions |
| CORS rule matching and response headers | `s3-core/src/cors.rs` | **None** - internal CorsIndex |
| Request validation (bucket names, keys) | `s3-core/src/validation.rs` | **None** - pure validation functions |
| Utility functions (ETags, version IDs) | `s3-core/src/utils.rs` | **None** |
| Configuration (S3Config, env vars) | `s3-core/src/config.rs` | **None** |
| Core types (AccountRegionStore) | `ruststack-core/` | **None** |
| Operation handler logic | `s3-core/src/ops/*.rs` | **Signature changes** - swap s3s types for our generated types |
| Integration tests | `tests/integration/` | **None** - test against HTTP API, not internal types |
| CI workflows | `.github/workflows/` | **None** |
| Docker setup | `Dockerfile`, `docker-compose.yml` | **None** |

### 11.1 Handler Refactoring Pattern

Each operation handler changes from:

```rust
// BEFORE (s3s-based)
use s3s::dto::*;
use s3s::{S3Request, S3Response, S3Result};

impl RustStackS3 {
    pub(crate) async fn handle_create_bucket(
        &self,
        req: S3Request<CreateBucketInput>,
    ) -> S3Result<S3Response<CreateBucketOutput>> {
        let input = req.input;
        let bucket_name = input.bucket;
        // ... business logic (unchanged) ...
        let output = CreateBucketOutput { location: Some(location) };
        Ok(S3Response::new(output))
    }
}
```

To:

```rust
// AFTER (our types)
use ruststack_s3_model::input::CreateBucketInput;
use ruststack_s3_model::output::CreateBucketOutput;
use ruststack_s3_model::error::S3Error;

impl RustStackS3 {
    pub(crate) async fn handle_create_bucket(
        &self,
        input: CreateBucketInput,
    ) -> Result<CreateBucketOutput, S3Error> {
        let bucket_name = &input.bucket;
        // ... business logic (UNCHANGED) ...
        Ok(CreateBucketOutput { location: Some(location) })
    }
}
```

The **business logic inside each handler stays the same**. Only the type imports and function signatures change.

---

## 12. What We Build New

| Component | Crate | Estimated LOC | Complexity |
|-----------|-------|--------------|------------|
| Smithy JSON → Rust type codegen tool | `codegen/` | 1,500-2,000 | Medium |
| S3 operation types (generated) | `ruststack-s3-model/` | ~5,000 (generated) | Low (automated) |
| XML serialization (generated + manual) | `ruststack-s3-xml/` | ~2,000 | Medium |
| SigV4 authentication | `ruststack-s3-auth/` | ~800 | Medium |
| HTTP router + virtual hosting | `ruststack-s3-http/router.rs` | ~400 | Medium |
| HTTP service (dispatch, middleware) | `ruststack-s3-http/service.rs` | ~500 | Medium |
| Request deserialization (per-operation) | `ruststack-s3-http/request.rs` | ~2,000 (generated) | Medium |
| Response serialization (per-operation) | `ruststack-s3-http/response.rs` | ~1,500 (generated) | Medium |
| S3 error formatting | `ruststack-s3-model/error.rs` | ~300 | Low |
| Response body type | `ruststack-s3-http/body.rs` | ~150 | Low |
| **Total new code** | | **~14,000** | |

Of this, ~8,500 LOC is auto-generated by our codegen tool.

---

## 13. Migration Plan

### Phase 1: Foundation (Week 1-2)

**Goal**: Set up the new crate structure, codegen tool, and generated types.

1. **Create `codegen/` tool**
   - Build the Smithy JSON parser in Rust
   - Download the S3 Smithy model from `aws/api-models-aws`
   - Generate `ruststack-s3-model` crate with all S3 types
   - Verify generated types compile

2. **Create `ruststack-s3-model` crate**
   - Commit generated types
   - Add `S3Operation` enum
   - Add `S3ErrorCode` and `S3Error` types
   - Write unit tests for key types

3. **Create `ruststack-s3-auth` crate**
   - Implement SigV4 signature verification
   - Implement presigned URL verification
   - Write unit tests with known test vectors from AWS docs

### Phase 2: HTTP Layer (Week 3-4)

**Goal**: Build the HTTP routing, serialization, and service layer.

4. **Create `ruststack-s3-xml` crate**
   - Implement `S3Serialize` and `S3Deserialize` traits
   - Implement XML serialization for all output types
   - Implement XML deserialization for all input types with XML bodies
   - Write unit tests against AWS SDK request/response samples

5. **Create `ruststack-s3-http` crate**
   - Implement `S3Router` (virtual hosting + operation identification)
   - Implement `S3HttpService` (hyper Service)
   - Implement `FromS3Request` for all operations (header/query/path/body extraction)
   - Implement `IntoS3Response` for all operations
   - Implement CORS middleware
   - Implement request ID generation

### Phase 3: Core Refactoring (Week 5-6)

**Goal**: Update `ruststack-s3-core` to use our types instead of s3s types.

6. **Refactor `ruststack-s3-core`**
   - Remove `s3s` and `s3s-aws` dependencies from `Cargo.toml`
   - Add `ruststack-s3-model` dependency
   - Update all `ops/*.rs` handler signatures (mechanical: swap type imports)
   - Update `provider.rs` (remove `impl s3s::S3`, add operation dispatch)
   - Verify all 242 unit tests pass with new types

7. **Refactor `ruststack-s3-server` (main.rs)**
   - Remove s3s service builder code
   - Use `S3HttpService` from `ruststack-s3-http`
   - Keep health check logic
   - Verify server starts and accepts requests

### Phase 4: Integration Testing (Week 7-8)

**Goal**: Validate the entire stack works end-to-end.

8. **Run integration tests**
   - Start server locally
   - Run all 45 integration tests
   - Fix any deserialization/routing issues
   - Compare behavior against the s3s-based version

9. **CI integration**
   - Update CI workflow to build the new stack
   - Ensure all tests pass
   - Commit, push, verify CI green

10. **Cleanup**
    - Remove s3s from workspace `Cargo.toml`
    - Remove any remaining s3s references
    - Update documentation

### Phase 5: Codegen Automation (Week 9+)

11. **Makefile target for regeneration**
    ```makefile
    generate-s3-types:
        cd codegen && cargo run -- \
            --model smithy-model/s3.json \
            --output ../crates/ruststack-s3-model/src/ \
            --xml-output ../crates/ruststack-s3-xml/src/ \
            --http-output ../crates/ruststack-s3-http/src/
        cargo +nightly fmt
    ```

12. **Model update workflow**
    - Fetch latest S3 model from `aws/api-models-aws`
    - Run codegen
    - Review diff
    - Run tests
    - Commit

---

## 14. Risk Analysis

### 14.1 Technical Risks

| Risk | Prob | Impact | Mitigation |
|------|------|--------|------------|
| **XML serialization edge cases** -- S3 has many XML quirks (`@s3UnwrappedXmlOutput`, flattened lists, empty elements vs absent elements) | High | High | Test against real AWS SDK request/response pairs. Use s3s's XML handling as reference. |
| **Operation routing ambiguity** -- Some S3 operations share the same HTTP method + path and differ only by query parameter presence or header | Medium | High | Build comprehensive routing table from Smithy model's `@http` traits. Test with the AWS SDK. |
| **SigV4 implementation correctness** -- Subtle canonicalization bugs can cause auth failures | Medium | Medium | Use AWS's published test vectors. Test against the real AWS SDK. Consider using `aws-sigv4` crate. |
| **Type incompatibility** -- Generated types may not perfectly match what our business logic expects | Medium | Medium | Phase 3 is specifically for addressing type mismatches. Most are mechanical (rename field, change Option wrapping). |
| **Codegen tool complexity** -- The S3 Smithy model has ~300 shapes; generating correct Rust for all of them is non-trivial | Medium | Medium | Start with hand-written types for the first 20 operations, then automate. |
| **Performance regression** -- Our HTTP layer might be slower than s3s's optimized path | Low | Low | s3s uses the same hyper/tower stack. Profile if needed. |

### 14.2 Schedule Risks

| Risk | Prob | Impact | Mitigation |
|------|------|--------|------------|
| **XML serialization takes longer than expected** due to S3 quirks | High | Medium | Budget extra time. Consider borrowing s3s's XML codegen patterns. |
| **Integration test failures reveal protocol issues** we didn't anticipate | High | Medium | Start integration testing early (Week 5-6) to surface issues sooner. |
| **Codegen tool is harder than expected** | Medium | Low | Fall back to hand-writing types for MVP. Automate later. |

### 14.3 Mitigation: Incremental Migration

The migration is designed to be incremental:
1. Both s3s-based and new crates can coexist in the workspace during transition
2. We can keep the s3s-based server running while building the new one
3. Integration tests validate both versions against the same HTTP API
4. If the new approach fails at any phase, we can fall back to s3s

---

## 15. Open Questions

### 15.1 Requires Decision Before Starting

1. **Codegen approach: Custom Rust parser vs smithy-rs Gradle?**
   - **Recommendation**: Custom Rust parser. ~2,000 LOC, no JVM dependency, full control.
   - Alternative: Run smithy-rs once, extract types, commit them. Simpler but requires JDK setup.

2. **SigV4: Build from scratch or use `aws-sigv4` crate?**
   - `aws-sigv4` is part of the official AWS SDK and handles all the edge cases.
   - Building our own gives full control but risks subtle bugs.
   - **Recommendation**: Use `aws-sigv4` crate for the signing key derivation and canonical request building. It's well-tested and maintained by AWS.

3. **Type generation scope: All 96 operations or incremental?**
   - Generating all types upfront means the codegen tool is validated against the full model.
   - Incremental means less risk per step but more integration work.
   - **Recommendation**: Generate all types upfront (it's mechanical). Implement handlers incrementally.

### 15.2 Can Be Decided During Implementation

4. **XML serialization: Per-type derive macro or codegen?**
5. **Whether to use `bytes::Bytes` or `Vec<u8>` for small XML bodies**
6. **Request body handling: Buffer small bodies, stream large ones, or always stream?**
7. **Error response customization: Include `<HostId>` element? `<BucketName>` for relevant errors?**

---

## Appendix A: Smithy JSON AST Structure (S3 Example)

The S3 Smithy model's JSON AST has this structure:

```json
{
  "smithy": "2.0",
  "shapes": {
    "com.amazonaws.s3#AmazonS3": {
      "type": "service",
      "version": "2006-03-01",
      "operations": [
        { "$ref": "com.amazonaws.s3#PutObject" },
        { "$ref": "com.amazonaws.s3#GetObject" },
        ...
      ]
    },
    "com.amazonaws.s3#PutObject": {
      "type": "operation",
      "input": { "$ref": "com.amazonaws.s3#PutObjectRequest" },
      "output": { "$ref": "com.amazonaws.s3#PutObjectOutput" },
      "errors": [...]
    },
    "com.amazonaws.s3#PutObjectRequest": {
      "type": "structure",
      "members": {
        "Bucket": {
          "target": "com.amazonaws.s3#BucketName",
          "traits": {
            "smithy.api#httpLabel": {},
            "smithy.api#required": {}
          }
        },
        "Key": {
          "target": "com.amazonaws.s3#ObjectKey",
          "traits": {
            "smithy.api#httpLabel": {},
            "smithy.api#required": {}
          }
        },
        "Body": {
          "target": "smithy.api#Blob",
          "traits": {
            "smithy.api#httpPayload": {}
          }
        },
        "ContentType": {
          "target": "smithy.api#String",
          "traits": {
            "smithy.api#httpHeader": "Content-Type"
          }
        },
        ...
      }
    }
  }
}
```

Our codegen tool reads this JSON and emits:

```rust
#[derive(Debug, Clone, Default)]
pub struct PutObjectInput {
    /// Required. Bucket name (from URI path).
    pub bucket: String,
    /// Required. Object key (from URI path).
    pub key: String,
    /// Object body (streaming, from HTTP payload).
    pub body: Option<StreamingBlob>,
    /// Content-Type (from HTTP header).
    pub content_type: Option<String>,
    // ...
}
```

## Appendix B: AWS SigV4 Test Vectors

AWS publishes test vectors for SigV4 at:
https://docs.aws.amazon.com/general/latest/gr/sigv4-signed-request-examples.html

Key test cases:
- GET request with no body
- POST request with query parameters
- Request with x-amz-date and Authorization headers
- Presigned URL with X-Amz-Signature in query
- Chunked upload with streaming signature

## Appendix C: S3 RestXml Protocol Reference

Key documents:
- [Smithy RestXml Protocol](https://smithy.io/2.0/aws/protocols/aws-restxml-protocol.html)
- [S3 Customizations](https://smithy.io/2.0/aws/customizations/s3-customizations.html)
- `noErrorWrapping: true` -- errors are `<Error><Code>...</Code>...` not wrapped in `<ErrorResponse>`
- `@s3UnwrappedXmlOutput` -- response bodies are not wrapped in operation name element
- XML namespace: `http://s3.amazonaws.com/doc/2006-03-01/`
