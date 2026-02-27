# s3s Crate Ecosystem Research

**Date:** 2026-02-26
**Purpose:** Comprehensive analysis of the s3s Rust crate for building S3-compatible services.

---

## Table of Contents

1. [Version Information](#1-version-information)
2. [S3 Trait Definition](#2-s3-trait-definition)
3. [Key Types](#3-key-types)
4. [s3s-fs Reference Implementation](#4-s3s-fs-reference-implementation)
5. [HTTP Server Setup with Hyper](#5-http-server-setup-with-hyper)
6. [Authentication (S3Auth)](#6-authentication-s3auth)
7. [Access Control (S3Access)](#7-access-control-s3access)
8. [Streaming Types (ByteStream)](#8-streaming-types-bytestream)
9. [Router and Service Setup](#9-router-and-service-setup)
10. [Custom Routes (S3Route)](#10-custom-routes-s3route)
11. [Virtual Host Parsing (S3Host)](#11-virtual-host-parsing-s3host)
12. [Configuration (S3Config)](#12-configuration-s3config)
13. [Related Crates](#13-related-crates)
14. [Breaking Changes Between Versions](#14-breaking-changes-between-versions)
15. [Complete Method Listing](#15-complete-method-listing)

---

## 1. Version Information

### Current Versions (as of February 2026)

| Crate | Latest Stable | Latest Pre-release | Published |
|-------|---------------|-------------------|-----------|
| s3s | 0.12.0 | 0.13.0-alpha.3 | 2026-02-08 |
| s3s-aws | 0.12.0 | 0.13.0-alpha.3 | 2026-02-08 |
| s3s-fs | 0.12.0 | 0.13.0-alpha.3 | 2026-02-08 |

### Version History

| Version | Date | MSRV |
|---------|------|------|
| 0.13.0-alpha.3 | 2026-02-08 | - |
| 0.13.0-alpha.2 | 2026-01-15 | - |
| 0.13.0-alpha.1 | 2026-01-14 | - |
| 0.12.0 | 2025-12-21 | 1.86.0 |
| 0.11.1 | 2025-10-05 | 1.85.0 |
| 0.11.0 | 2025-03-28 | 1.85.0 |

### Repository

- **GitHub:** https://github.com/Nugine/s3s (also at https://github.com/s3s-project/s3s)
- **License:** Apache-2.0
- **Total Downloads:** ~330,000+

---

## 2. S3 Trait Definition

The `S3` trait is the core abstraction. It is auto-generated from AWS Smithy models and contains **96 async methods**, all of which are **provided (default) methods** that return `S3Error::NotImplemented`. You only override the methods you need.

### Trait Bounds

```rust
#[async_trait::async_trait]
pub trait S3: Send + Sync + 'static {
    // 96 provided methods...
}
```

### Method Signature Pattern

Every method follows this exact pattern:

```rust
async fn operation_name(
    &self,
    _req: S3Request<OperationNameInput>,
) -> S3Result<S3Response<OperationNameOutput>> {
    Err(s3_error!(NotImplemented, "OperationName is not implemented yet"))
}
```

For example:

```rust
async fn get_object(
    &self,
    _req: S3Request<GetObjectInput>,
) -> S3Result<S3Response<GetObjectOutput>> {
    Err(s3_error!(NotImplemented, "GetObject is not implemented yet"))
}

async fn put_object(
    &self,
    _req: S3Request<PutObjectInput>,
) -> S3Result<S3Response<PutObjectOutput>> {
    Err(s3_error!(NotImplemented, "PutObject is not implemented yet"))
}
```

### Minimal Implementation

Because all methods have defaults, a minimal implementation is just:

```rust
struct MockS3;
impl S3 for MockS3 {}
```

This compiles and returns `NotImplemented` for every operation.

### Note on async_trait

The S3 trait uses `#[async_trait::async_trait]` (the crate, not native async traits) because it requires object safety (`Arc<dyn S3>`). The actual method signatures use `Pin<Box<dyn Future<...> + Send + 'async_trait>>` under the hood.

---

## 3. Key Types

### S3Request<T>

```rust
#[derive(Debug, Clone)]
pub struct S3Request<T> {
    /// S3 operation input
    pub input: T,
    /// HTTP method
    pub method: Method,
    /// HTTP URI
    pub uri: Uri,
    /// HTTP headers
    pub headers: HeaderMap,
    /// Request extensions (pass custom data between middlewares)
    pub extensions: Extensions,
    /// S3 identity information. None means anonymous request.
    pub credentials: Option<Credentials>,
    /// S3 requested region
    pub region: Option<Region>,
    /// S3 requested service
    pub service: Option<String>,
    /// Streaming trailers handle for AWS SigV4 streaming with trailers
    pub trailing_headers: Option<TrailingHeaders>,
}

impl<T> S3Request<T> {
    pub fn map_input<U>(self, f: impl FnOnce(T) -> U) -> S3Request<U>;
}
```

### S3Response<T>

```rust
#[derive(Debug, Clone)]
pub struct S3Response<T> {
    /// S3 operation output
    pub output: T,
    /// HTTP status code override
    pub status: Option<StatusCode>,
    /// HTTP headers override
    pub headers: HeaderMap,
    /// Response extensions
    pub extensions: Extensions,
}

impl<T> S3Response<T> {
    pub fn new(output: T) -> Self;
    pub fn with_status(output: T, status: StatusCode) -> Self;
    pub fn with_headers(output: T, headers: HeaderMap) -> Self;
    pub fn map_output<U>(self, f: impl FnOnce(T) -> U) -> S3Response<U>;
}
```

### S3Error

```rust
#[derive(Debug, thiserror::Error)]
pub struct S3Error(Box<Inner>);

impl S3Error {
    // Constructors
    pub fn new(code: S3ErrorCode) -> Self;
    pub fn with_message(code: S3ErrorCode, msg: impl Into<Cow<'static, str>>) -> Self;
    pub fn with_source(code: S3ErrorCode, source: StdError) -> Self;
    pub fn internal_error<E: std::error::Error + Send + Sync + 'static>(source: E) -> Self;

    // Setters
    pub fn set_code(&mut self, val: S3ErrorCode);
    pub fn set_message(&mut self, val: impl Into<Cow<'static, str>>);
    pub fn set_request_id(&mut self, val: impl Into<String>);
    pub fn set_source(&mut self, val: StdError);
    pub fn set_status_code(&mut self, val: StatusCode);
    pub fn set_headers(&mut self, val: HeaderMap);

    // Getters
    pub fn code(&self) -> &S3ErrorCode;
    pub fn message(&self) -> Option<&str>;
    pub fn request_id(&self) -> Option<&str>;
    pub fn source(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)>;
    pub fn status_code(&self) -> Option<StatusCode>;
    pub fn headers(&self) -> Option<&HeaderMap>;

    /// Serialize to HTTP response (XML error body)
    pub fn to_http_response(self) -> S3Result<HttpResponse>;
}
```

### s3_error! Macro

Convenient macro for creating S3Error instances:

```rust
// Just error code
s3_error!(NoSuchKey)

// Error code with message
s3_error!(NoSuchKey, "The specified key does not exist")

// Error code with formatted message
s3_error!(NoSuchKey, "Key {} does not exist", key)

// Error with source
s3_error!(io_err, NoSuchKey)

// Error with source and message
s3_error!(io_err, NoSuchKey, "Failed to read key {}", key)
```

### S3Result

```rust
pub type S3Result<T = (), E = S3Error> = std::result::Result<T, E>;
```

### S3ErrorCode

An enum with all standard S3 error codes plus a `Custom(String)` variant. Key codes include:

- `NoSuchBucket`, `NoSuchKey`, `NoSuchUpload`, `NoSuchVersion`
- `BucketAlreadyExists`, `BucketAlreadyOwnedByYou`, `BucketNotEmpty`
- `AccessDenied`, `InvalidAccessKeyId`
- `NotImplemented`, `InternalError`
- `InvalidArgument`, `InvalidRange`, `InvalidPart`, `InvalidPartOrder`
- `MalformedXML`, `PreconditionFailed`, `NotModified`
- `EntityTooSmall`, `EntityTooLarge`, `BadDigest`
- `Custom(String)` for custom error codes

### HttpRequest / HttpResponse / HttpError

```rust
pub type HttpRequest<B = Body> = http::Request<B>;
pub type HttpResponse<B = Body> = http::Response<B>;

/// An error that indicates a failure of an HTTP request.
/// Passing this error to hyper will cause it to abort the connection.
#[derive(Debug)]
pub struct HttpError(StdError);

impl HttpError {
    pub fn new(err: StdError) -> Self;
}
```

### Body

The `Body` type wraps HTTP bodies and supports multiple internal representations:

```rust
pub struct Body { /* private fields */ }

impl Body {
    pub fn empty() -> Self;
    pub fn http_body<B>(body: B) -> Self;       // Sync boxed body
    pub fn http_body_unsync<B>(body: B) -> Self; // Unsync boxed body
}

impl From<String> for Body { ... }
impl From<Bytes> for Body { ... }
impl From<hyper::body::Incoming> for Body { ... }
```

Body implements `http_body::Body<Data = Bytes>`.

### TrailingHeaders

```rust
#[derive(Clone)]
pub struct TrailingHeaders(Arc<Mutex<Option<HeaderMap>>>);

impl TrailingHeaders {
    pub fn is_ready(&self) -> bool;
    pub fn take(&self) -> Option<HeaderMap>;  // One-shot take
    pub fn read<R>(&self, f: impl FnOnce(&HeaderMap) -> R) -> Option<R>;
}
```

### S3Operation

Describes the S3 operation being invoked. Available from the access context.

### StdError

```rust
pub type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;
```

---

## 4. s3s-fs Reference Implementation

`s3s-fs` is the reference filesystem-based S3 implementation. It implements ~20 S3 operations.

### FileSystem Struct

```rust
#[derive(Debug)]
pub struct FileSystem {
    root: PathBuf,
    tmp_file_counter: AtomicU64,
}

impl FileSystem {
    pub fn new(root: impl AsRef<Path>) -> Result<Self>;
}
```

### Operations Implemented by s3s-fs

| Operation | Notes |
|-----------|-------|
| create_bucket | Creates directory at root |
| delete_bucket | Removes directory recursively |
| head_bucket | Checks directory existence |
| list_buckets | Lists directories at root |
| put_object | Writes file with MD5/checksum |
| get_object | Reads file with Range support |
| head_object | File metadata |
| delete_object | Removes file |
| delete_objects | Batch file removal |
| copy_object | File copy with metadata |
| list_objects | Directory listing with prefix/delimiter |
| list_objects_v2 | Enhanced directory listing |
| get_bucket_location | Returns default location |
| create_multipart_upload | UUID-based upload tracking |
| upload_part | Part file write with checksums |
| upload_part_copy | Copy byte range as part |
| complete_multipart_upload | Part assembly with composite ETag |
| abort_multipart_upload | Part cleanup |
| list_parts | List uploaded parts |

### Implementation Pattern (PutObject Example)

```rust
#[async_trait::async_trait]
impl S3 for FileSystem {
    #[tracing::instrument]
    async fn put_object(
        &self,
        req: S3Request<PutObjectInput>,
    ) -> S3Result<S3Response<PutObjectOutput>> {
        let input = req.input;
        let path = self.get_object_path(&input.bucket, &input.key)?;

        // Ensure parent directory exists
        if let Some(dir_path) = path.parent() {
            try_!(fs::create_dir_all(&dir_path).await);
        }

        // Write object data from streaming body
        let PutObjectParts { body, .. } = input.take_parts();
        let body = body.ok_or_else(|| s3_error!(IncompleteBody))?;

        // Write file and compute checksums
        let (md5_hash, file_len) = self.write_file(&path, body).await?;

        // Save metadata
        // ... (user metadata, content-type, etc.)

        let output = PutObjectOutput {
            e_tag: Some(ETag::Strong(hex(&md5_hash))),
            ..Default::default()
        };
        Ok(S3Response::new(output))
    }
}
```

### Implementation Pattern (GetObject Example)

```rust
async fn get_object(
    &self,
    req: S3Request<GetObjectInput>,
) -> S3Result<S3Response<GetObjectOutput>> {
    let input = req.input;
    let object_path = self.get_object_path(&input.bucket, &input.key)?;

    let mut file = fs::File::open(&object_path)
        .await
        .map_err(|e| s3_error!(e, NoSuchKey))?;

    let file_metadata = try_!(file.metadata().await);
    let last_modified = Timestamp::from(try_!(file_metadata.modified()));
    let file_len = file_metadata.len();

    // Handle Range requests
    let (content_length, content_range) = match input.range {
        None => (file_len, None),
        Some(range) => {
            let file_range = range.check(file_len)?;
            let content_length = file_range.end - file_range.start;
            let content_range = fmt_content_range(
                file_range.start, file_range.end - 1, file_len
            );
            (content_length, Some(content_range))
        }
    };

    // Create streaming body from file reader
    let body = bytes_stream(
        ReaderStream::with_capacity(file, 4096),
        content_length_usize,
    );

    let output = GetObjectOutput {
        body: Some(StreamingBlob::wrap(body)),
        content_length: Some(content_length_i64),
        content_range,
        last_modified: Some(last_modified),
        e_tag: Some(ETag::Strong(md5_sum)),
        ..Default::default()
    };
    Ok(S3Response::new(output))
}
```

---

## 5. HTTP Server Setup with Hyper

### Complete Server Example (from s3s-fs main.rs)

```rust
use s3s::auth::SimpleAuth;
use s3s::host::MultiDomain;
use s3s::service::S3ServiceBuilder;

use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as ConnBuilder;
use tokio::net::TcpListener;

#[tokio::main]
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create your S3 implementation
    let fs = FileSystem::new("/path/to/root")?;

    // 2. Build the S3 service
    let service = {
        let mut b = S3ServiceBuilder::new(fs);

        // Optional: Enable authentication
        b.set_auth(SimpleAuth::from_single("access_key", "secret_key"));

        // Optional: Enable virtual-hosted-style requests
        b.set_host(MultiDomain::new(&["s3.example.com"])?);

        b.build()
    };

    // 3. Start the TCP listener
    let listener = TcpListener::bind("127.0.0.1:8014").await?;
    let local_addr = listener.local_addr()?;

    // 4. Create the HTTP server (supports HTTP/1.1 and HTTP/2)
    let http_server = ConnBuilder::new(TokioExecutor::new());
    let graceful = hyper_util::server::graceful::GracefulShutdown::new();

    let mut ctrl_c = std::pin::pin!(tokio::signal::ctrl_c());

    println!("server is running at http://{local_addr}");

    // 5. Accept connections
    loop {
        let (socket, _) = tokio::select! {
            res = listener.accept() => {
                match res {
                    Ok(conn) => conn,
                    Err(err) => {
                        tracing::error!("error accepting connection: {err}");
                        continue;
                    }
                }
            }
            _ = ctrl_c.as_mut() => {
                break;
            }
        };

        let conn = http_server.serve_connection(
            TokioIo::new(socket),
            service.clone(),
        );
        let conn = graceful.watch(conn.into_owned());
        tokio::spawn(async move {
            let _ = conn.await;
        });
    }

    // 6. Graceful shutdown
    tokio::select! {
        () = graceful.shutdown() => {
            tracing::debug!("Gracefully shutdown!");
        },
        () = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
            tracing::debug!("Waited 10 seconds for graceful shutdown, aborting...");
        }
    }

    Ok(())
}
```

### Service Compatibility

`S3Service` implements both:
- `hyper::service::Service<http::Request<hyper::body::Incoming>>` - Direct hyper integration
- `tower::Service<http::Request<B>>` where `B: http_body::Body<Data = Bytes>` - Tower/Axum integration

`S3Service` is `Clone` (cheap, uses `Arc` internally).

---

## 6. Authentication (S3Auth)

### S3Auth Trait

```rust
#[async_trait::async_trait]
pub trait S3Auth: Send + Sync + 'static {
    /// Gets the corresponding secret key for the given access key.
    async fn get_secret_key(&self, access_key: &str) -> S3Result<SecretKey>;
}
```

The trait has a single method. s3s handles all SigV4/SigV2 signature verification internally -- you just need to provide the secret key lookup.

### SecretKey Type

```rust
#[derive(Clone)]
pub struct SecretKey(Box<str>);

impl SecretKey {
    pub fn expose(&self) -> &str;  // Get the secret key value
}

// Security:
impl Zeroize for SecretKey { ... }      // Zeroed on drop
impl Drop for SecretKey { ... }          // Calls zeroize
impl ConstantTimeEq for SecretKey { ... } // Constant-time comparison
impl Debug for SecretKey { ... }          // Prints "[SENSITIVE-SECRET-KEY]"
impl Serialize for SecretKey { ... }      // Serializes as placeholder

// Constructors:
impl From<String> for SecretKey { ... }
impl From<&str> for SecretKey { ... }
impl From<Box<str>> for SecretKey { ... }
```

### Credentials Type

```rust
#[derive(Debug, Clone)]
pub struct Credentials {
    pub access_key: String,
    pub secret_key: SecretKey,
}
```

### SimpleAuth (Built-in)

```rust
#[derive(Debug, Default)]
pub struct SimpleAuth {
    map: HashMap<String, SecretKey>,
}

impl SimpleAuth {
    pub fn new() -> Self;
    pub fn from_single(access_key: impl Into<String>, secret_key: impl Into<SecretKey>) -> Self;
    pub fn register(&mut self, access_key: String, secret_key: SecretKey) -> Option<SecretKey>;
    pub fn lookup(&self, access_key: &str) -> Option<&SecretKey>;
}

#[async_trait::async_trait]
impl S3Auth for SimpleAuth {
    async fn get_secret_key(&self, access_key: &str) -> S3Result<SecretKey> {
        match self.lookup(access_key) {
            None => Err(s3_error!(NotSignedUp, "Your account is not signed up")),
            Some(s) => Ok(s.clone()),
        }
    }
}
```

### Custom Auth Example

```rust
struct DatabaseAuth {
    pool: PgPool,
}

#[async_trait::async_trait]
impl S3Auth for DatabaseAuth {
    async fn get_secret_key(&self, access_key: &str) -> S3Result<SecretKey> {
        let row = sqlx::query("SELECT secret_key FROM credentials WHERE access_key = $1")
            .bind(access_key)
            .fetch_optional(&self.pool)
            .await
            .map_err(S3Error::internal_error)?;

        match row {
            Some(row) => Ok(SecretKey::from(row.get::<String, _>("secret_key"))),
            None => Err(s3_error!(InvalidAccessKeyId)),
        }
    }
}
```

### Auth Behavior

- If **no auth** is set on the builder: unsigned requests are allowed, but signed requests fail with `NotImplemented`
- If **auth is set**: s3s verifies SigV4/SigV2 signatures automatically by calling `get_secret_key`
- Supports presigned URLs (with configurable clock skew tolerance)
- Supports STS signature validation (v0.12.0+)

---

## 7. Access Control (S3Access)

### S3Access Trait (v0.11.0+)

```rust
#[async_trait::async_trait]
pub trait S3Access: Send + Sync + 'static {
    async fn check(&self, cx: &mut S3AccessContext<'_>) -> S3Result<()>;

    // Per-operation methods also available (e.g., get_object, put_object)
    // Generated from Smithy model with default implementations that call check()
}
```

### S3AccessContext

```rust
pub struct S3AccessContext<'a> {
    // Provides:
    pub fn credentials(&self) -> Option<&Credentials>;
    pub fn s3_op(&self) -> &S3Operation;
    pub fn s3_path(&self) -> &S3Path; // bucket + key
    // ... and more
}
```

### Important: Auth + Access Interaction

Access checks are **only enforced when auth is configured**. Setting `set_access()` alone does nothing:

```rust
let mut builder = S3ServiceBuilder::new(MyS3);
// MUST set auth for access checks to work
builder.set_auth(SimpleAuth::from_single("AK", "SK"));
builder.set_access(MyAccessControl);
let service = builder.build();
```

When auth is configured but no custom access provider is set, the **default access check** allows authenticated requests and denies anonymous ones:

```rust
fn default_check(cx: &mut S3AccessContext<'_>) -> S3Result<()> {
    match cx.credentials() {
        Some(_) => Ok(()),
        None => Err(s3_error!(AccessDenied, "Signature is required")),
    }
}
```

---

## 8. Streaming Types (ByteStream)

### ByteStream Trait

```rust
pub trait ByteStream: Stream {
    fn remaining_length(&self) -> RemainingLength {
        RemainingLength::unknown()
    }
}
```

### DynByteStream

```rust
pub type DynByteStream = Pin<Box<
    dyn ByteStream<Item = Result<Bytes, StdError>> + Send + Sync + 'static
>>;
```

### RemainingLength

```rust
pub struct RemainingLength {
    lower: usize,
    upper: Option<usize>,
}

impl RemainingLength {
    pub fn new(lower: usize, upper: Option<usize>) -> Self;
    pub fn unknown() -> Self;          // lower=0, upper=None
    pub fn new_exact(n: usize) -> Self; // lower=n, upper=Some(n)
    pub fn exact(&self) -> Option<usize>;
}

// Conversions:
impl From<RemainingLength> for http_body::SizeHint { ... }
impl From<http_body::SizeHint> for RemainingLength { ... }
```

### StreamingBlob (in dto)

Used in operations like GetObject and PutObject for streaming data:

```rust
// In GetObjectOutput:
pub body: Option<StreamingBlob>,

// In PutObjectInput:
pub body: Option<StreamingBlob>,
```

`StreamingBlob` wraps a `DynByteStream` and provides `StreamingBlob::wrap(stream)`.

---

## 9. Router and Service Setup

### S3ServiceBuilder

```rust
pub struct S3ServiceBuilder {
    s3: Arc<dyn S3>,
    config: Option<Arc<dyn S3ConfigProvider>>,
    host: Option<Box<dyn S3Host>>,
    auth: Option<Box<dyn S3Auth>>,
    access: Option<Box<dyn S3Access>>,
    route: Option<Box<dyn S3Route>>,
    validation: Option<Box<dyn NameValidation>>,
}

impl S3ServiceBuilder {
    pub fn new(s3: impl S3) -> Self;
    pub fn set_config(&mut self, config: Arc<dyn S3ConfigProvider>);
    pub fn set_host(&mut self, host: impl S3Host);
    pub fn set_auth(&mut self, auth: impl S3Auth);
    pub fn set_access(&mut self, access: impl S3Access);
    pub fn set_route(&mut self, route: impl S3Route);
    pub fn set_validation(&mut self, validation: impl NameValidation);
    pub fn build(self) -> S3Service;
}
```

### S3Service

```rust
#[derive(Clone)]
pub struct S3Service {
    inner: Arc<Inner>,
}

impl S3Service {
    pub async fn call(&self, req: HttpRequest) -> Result<HttpResponse, HttpError>;
}

// Implements:
impl hyper::service::Service<http::Request<hyper::body::Incoming>> for S3Service { ... }
impl<B> tower::Service<http::Request<B>> for S3Service { ... }
```

### Minimal Setup

```rust
use s3s::service::S3ServiceBuilder;

let service = S3ServiceBuilder::new(MyS3).build();
```

### Full Configuration

```rust
use s3s::service::S3ServiceBuilder;
use s3s::auth::SimpleAuth;
use s3s::host::MultiDomain;
use s3s::config::{S3Config, StaticConfigProvider, HotReloadConfigProvider};
use std::sync::Arc;

let mut builder = S3ServiceBuilder::new(MyS3);

// Authentication
builder.set_auth(SimpleAuth::from_single("ACCESS_KEY", "SECRET_KEY"));

// Virtual-hosted style domains
builder.set_host(MultiDomain::new(&[
    "s3.localhost.localstack.cloud",
    "s3.us-east-1.localhost.localstack.cloud",
])?);

// Custom configuration
let mut config = S3Config::default();
config.xml_max_body_size = 10 * 1024 * 1024;
builder.set_config(Arc::new(StaticConfigProvider::new(Arc::new(config))));

// Custom access control
builder.set_access(MyAccessControl);

// Custom routes (e.g., health check)
builder.set_route(MyCustomRoute);

// Custom name validation
builder.set_validation(MyNameValidation);

let service = builder.build();
```

### Request Processing Pipeline

1. Parse HTTP request
2. Parse Host header (if S3Host configured)
3. Check custom route match (if S3Route configured)
4. Authenticate request (if S3Auth configured) - verify SigV4/V2
5. Authorize request (if auth configured) - call S3Access::check
6. Route to S3 operation based on method + path + query params
7. Deserialize HTTP body to operation input DTO
8. Call the S3 trait method
9. Serialize operation output to HTTP response
10. Return HTTP response

---

## 10. Custom Routes (S3Route)

### S3Route Trait

```rust
#[async_trait::async_trait]
pub trait S3Route: Send + Sync + 'static {
    /// Check if this route matches the request
    fn is_match(
        &self,
        method: &Method,
        uri: &Uri,
        headers: &HeaderMap,
        extensions: &mut Extensions,
    ) -> bool;

    /// Check access for this route (default: require credentials)
    async fn check_access(&self, req: &mut S3Request<Body>) -> S3Result<()> {
        match req.credentials {
            Some(_) => Ok(()),
            None => Err(s3_error!(AccessDenied, "Signature is required")),
        }
    }

    /// Handle the request
    async fn call(&self, req: S3Request<Body>) -> S3Result<S3Response<Body>>;
}
```

### Health Check Route Example

```rust
#[derive(Clone)]
struct HealthCheckRoute;

#[async_trait::async_trait]
impl S3Route for HealthCheckRoute {
    fn is_match(
        &self,
        method: &Method,
        uri: &Uri,
        _headers: &HeaderMap,
        _extensions: &mut Extensions,
    ) -> bool {
        method == Method::GET && uri.path() == "/health"
    }

    // Allow unauthenticated health checks
    async fn check_access(&self, _req: &mut S3Request<Body>) -> S3Result<()> {
        Ok(())
    }

    async fn call(&self, _req: S3Request<Body>) -> S3Result<S3Response<Body>> {
        Ok(S3Response::new(Body::from("OK".to_string())))
    }
}
```

Routes are checked **before** the standard S3 operation routing. If `is_match` returns `true`, the custom route handles the request instead.

---

## 11. Virtual Host Parsing (S3Host)

### S3Host Trait

```rust
pub trait S3Host: Send + Sync + 'static {
    fn parse_host_header<'a>(&'a self, host: &'a str) -> S3Result<VirtualHost<'a>>;
}
```

### VirtualHost

```rust
#[derive(Debug, Clone)]
pub struct VirtualHost<'a> {
    domain: Cow<'a, str>,
    bucket: Option<Cow<'a, str>>,
    region: Option<Cow<'a, str>>,
}

impl<'a> VirtualHost<'a> {
    pub fn new(domain: impl Into<Cow<'a, str>>) -> Self;
    pub fn with_bucket(mut self, bucket: impl Into<Cow<'a, str>>) -> Self;
    pub fn with_region(mut self, region: impl Into<Cow<'a, str>>) -> Self;
    pub fn domain(&self) -> &str;
    pub fn bucket(&self) -> Option<&str>;
    pub fn region(&self) -> Option<&str>;
}
```

### Built-in Implementations

**SingleDomain:** For single base domain

```rust
let host = SingleDomain::new("s3.example.com")?;
// "s3.example.com"           -> domain="s3.example.com", bucket=None
// "mybucket.s3.example.com"  -> domain="s3.example.com", bucket=Some("mybucket")
```

**MultiDomain:** For multiple base domains

```rust
let host = MultiDomain::new(&[
    "s3.localhost.localstack.cloud",
    "s3.us-east-1.amazonaws.com",
])?;
```

---

## 12. Configuration (S3Config)

### S3Config Struct

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct S3Config {
    /// Maximum size for XML body payloads (default: 20 MB)
    pub xml_max_body_size: usize,
    /// Maximum file size for POST object (default: 5 GB)
    pub post_object_max_file_size: u64,
    /// Maximum size per form field (default: 1 MB)
    pub form_max_field_size: usize,
    /// Maximum total size for all form fields (default: 20 MB)
    pub form_max_fields_size: usize,
    /// Maximum number of parts in multipart form (default: 1000)
    pub form_max_parts: usize,
    /// Maximum allowed time skew for presigned URLs (default: 900s / 15 min)
    pub presigned_url_max_skew_time_secs: u32,
}
```

### Config Providers

```rust
pub trait S3ConfigProvider: Send + Sync + 'static {
    fn snapshot(&self) -> Arc<S3Config>;
}
```

**StaticConfigProvider:** Immutable config (default if not set)

```rust
let config = Arc::new(StaticConfigProvider::new(Arc::new(S3Config {
    xml_max_body_size: 10 * 1024 * 1024,
    ..Default::default()
})));
builder.set_config(config);
```

**HotReloadConfigProvider:** Runtime-updatable config (uses `ArcSwap`)

```rust
let hot_config = Arc::new(HotReloadConfigProvider::default());
builder.set_config(hot_config.clone());

// Later, update config at runtime:
hot_config.update(Arc::new(S3Config {
    xml_max_body_size: 30 * 1024 * 1024,
    ..Default::default()
}));
```

---

## 13. Related Crates

### s3s (core)

The core crate providing:
- `S3` trait (96 operations)
- HTTP/S3 protocol handling (XML, headers, routing)
- SigV4/V2 signature verification
- `S3Service` and `S3ServiceBuilder`
- All DTO types (generated from Smithy)
- Streaming support
- Checksum algorithms

### s3s-fs

Reference filesystem implementation:
- Implements ~20 core S3 operations
- Stores objects as files, buckets as directories
- Metadata stored as JSON sidecar files
- Includes a CLI binary with auth, virtual-host, and TLS support
- Good reference for implementing your own S3 backend

### s3s-aws

AWS SDK integration:
- `Proxy` type - proxies S3 requests to real AWS S3 via aws-sdk-s3
- `conv` module - type conversions between s3s DTOs and aws-sdk-s3 types
- `Connector` and `Client` for AWS SDK HTTP client integration

### s3s-proxy (binary)

A standalone binary that proxies requests to AWS S3. Uses `s3s-aws::Proxy`.

### s3s-test

Custom test framework for S3 compatibility testing:
- Test runner infrastructure
- CLI for running tests
- Report generation

### s3s-e2e

End-to-end tests:
- Comprehensive S3 compatibility test suite
- Tests multipart uploads, checksums, advanced features
- Uses aws-sdk-s3 as the client

### s3s-model

S3 model definitions (shared between codegen and runtime).

### s3s-policy

S3 policy language:
- Grammar model types for serialization/deserialization
- `PatternSet` for matching multiple patterns

### s3s-wasm

WebAssembly support for running s3s in WASM environments.

---

## 14. Breaking Changes Between Versions

### v0.10.x -> v0.11.0 (March 2025)

**Architecture:**
- **S3Access trait added** - `S3Auth::check_access` moved to `S3Access::check`
- **S3Host trait added** - `S3ServiceBuilder::set_base_domain` replaced with `set_host`
- **S3Route trait added** - Custom route support
- **DTO updates** from latest AWS Smithy model (type changes may require migration)
- New precondition and write_offset_bytes parameters accepted

**Migration:**
- Replace `builder.set_base_domain("domain")` with `builder.set_host(SingleDomain::new("domain")?)`
- Move any `check_access` logic from `S3Auth` to a new `S3Access` implementation

### v0.11.x -> v0.12.0 (December 2025)

**Architecture refactoring:**
- `S3Service` made shared (now `Clone` with `Arc` internals)
- HTTP types moved to protocol module (`HttpRequest`, `HttpResponse`, `HttpError` re-exported from `protocol`)
- Better route definitions
- Protocol types moved

**Security improvements:**
- Memory allocation limits added to stream parsers
- Unbounded memory allocation fixes in http::body and POST object
- `SecretKey::PartialEq` removed, replaced with `ConstantTimeEq` from `subtle`

**Typed ETag:**
- Strongly-typed `ETag` type replacing `String`
- `ETagCondition` type for If-Match/If-None-Match wildcards
- RFC 9110-compliant ETag comparison

**Configuration:**
- All configuration types now implement `Default`, `Serialize`, `Deserialize`
- `S3Config` is `#[non_exhaustive]`

**Other:**
- Content-Type validation changes (allow custom, allow empty)
- AWS STS signature validation support
- SigV2 POST signature support
- Enhanced checksum support with streaming trailers
- Custom validation via `S3ServiceBuilder::set_validation`

### v0.12.0 -> v0.13.0-alpha (January-February 2026)

Still in alpha. Check the main branch on GitHub for latest changes. Docker images migrated from Docker Hub to GHCR.

---

## 15. Complete Method Listing

All 96 methods in the S3 trait (alphabetical):

```
abort_multipart_upload
complete_multipart_upload
copy_object
create_bucket
create_bucket_metadata_table_configuration
create_multipart_upload
delete_bucket
delete_bucket_analytics_configuration
delete_bucket_cors
delete_bucket_encryption
delete_bucket_intelligent_tiering_configuration
delete_bucket_inventory_configuration
delete_bucket_lifecycle
delete_bucket_metadata_table_configuration
delete_bucket_metrics_configuration
delete_bucket_ownership_controls
delete_bucket_policy
delete_bucket_replication
delete_bucket_tagging
delete_bucket_website
delete_object
delete_object_tagging
delete_objects
delete_public_access_block
get_bucket_accelerate_configuration
get_bucket_acl
get_bucket_analytics_configuration
get_bucket_cors
get_bucket_encryption
get_bucket_intelligent_tiering_configuration
get_bucket_inventory_configuration
get_bucket_lifecycle_configuration
get_bucket_location
get_bucket_logging
get_bucket_metadata_table_configuration
get_bucket_metrics_configuration
get_bucket_notification_configuration
get_bucket_ownership_controls
get_bucket_policy
get_bucket_policy_status
get_bucket_replication
get_bucket_request_payment
get_bucket_tagging
get_bucket_versioning
get_bucket_website
get_object
get_object_acl
get_object_attributes
get_object_legal_hold
get_object_lock_configuration
get_object_retention
get_object_tagging
get_object_torrent
get_public_access_block
head_bucket
head_object
list_bucket_analytics_configurations
list_bucket_intelligent_tiering_configurations
list_bucket_inventory_configurations
list_bucket_metrics_configurations
list_buckets
list_multipart_uploads
list_object_versions
list_objects
list_objects_v2
list_parts
post_object
put_bucket_accelerate_configuration
put_bucket_acl
put_bucket_analytics_configuration
put_bucket_cors
put_bucket_encryption
put_bucket_intelligent_tiering_configuration
put_bucket_inventory_configuration
put_bucket_lifecycle_configuration
put_bucket_logging
put_bucket_metrics_configuration
put_bucket_notification_configuration
put_bucket_ownership_controls
put_bucket_policy
put_bucket_replication
put_bucket_request_payment
put_bucket_tagging
put_bucket_versioning
put_bucket_website
put_object
put_object_acl
put_object_legal_hold
put_object_lock_configuration
put_object_retention
put_object_tagging
put_public_access_block
restore_object
select_object_content
upload_part
upload_part_copy
write_get_object_response
```

---

## Key Dependencies of s3s (v0.12.0)

| Dependency | Purpose |
|-----------|---------|
| async-trait | Object-safe async traits for S3/S3Auth/S3Access/S3Route |
| hyper | HTTP server integration |
| tower | Service trait compatibility |
| http / http-body / http-body-util | HTTP types |
| bytes | Zero-copy byte buffers |
| futures | Stream trait, async utilities |
| quick-xml | XML serialization/deserialization |
| serde / serde_json | JSON serialization |
| hmac / sha2 / md-5 / sha1 | Cryptographic operations for SigV4 |
| crc-fast | CRC32/CRC32C/CRC64NVME checksums |
| base64-simd / hex-simd | Fast encoding |
| subtle | Constant-time comparison |
| zeroize | Secure memory clearing |
| arc-swap | Lock-free shared config updates |
| tracing | Structured logging |
| thiserror | Error type derivation |
| tokio | Async runtime (time features) |
| pin-project-lite | Safe pin projections |
| nom | Parser combinators (for HTTP parsing) |
| chrono / time | Timestamp handling |

---

## Sources

- [s3s on crates.io](https://crates.io/crates/s3s) - Version 0.12.0 / 0.13.0-alpha.3
- [s3s on docs.rs](https://docs.rs/s3s/0.12.0/s3s/) - API documentation
- [s3s GitHub Repository](https://github.com/Nugine/s3s) - Source code
- [s3s CHANGELOG](https://github.com/Nugine/s3s/blob/main/CHANGELOG.md) - Version history
- Source files examined directly via GitHub API:
  - `crates/s3s/src/s3_trait.rs` - S3 trait definition
  - `crates/s3s/src/protocol.rs` - S3Request, S3Response, HttpError
  - `crates/s3s/src/error/mod.rs` - S3Error, S3Result, s3_error! macro
  - `crates/s3s/src/stream.rs` - ByteStream, DynByteStream, RemainingLength
  - `crates/s3s/src/service.rs` - S3Service, S3ServiceBuilder
  - `crates/s3s/src/auth/mod.rs` - S3Auth trait
  - `crates/s3s/src/auth/secret_key.rs` - SecretKey, Credentials
  - `crates/s3s/src/auth/simple_auth.rs` - SimpleAuth
  - `crates/s3s/src/access/mod.rs` - S3Access trait
  - `crates/s3s/src/route.rs` - S3Route trait
  - `crates/s3s/src/host.rs` - S3Host, SingleDomain, MultiDomain
  - `crates/s3s/src/config.rs` - S3Config, S3ConfigProvider
  - `crates/s3s/src/http/body.rs` - Body type
  - `crates/s3s/src/lib.rs` - Top-level module exports
  - `crates/s3s-fs/src/s3.rs` - FileSystem S3 trait implementation
  - `crates/s3s-fs/src/main.rs` - Server setup example
  - `crates/s3s-fs/src/fs.rs` - FileSystem struct
  - `crates/s3s-aws/src/lib.rs` - s3s-aws exports
  - `crates/s3s/Cargo.toml` - Dependencies
