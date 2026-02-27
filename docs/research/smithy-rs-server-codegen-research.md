# smithy-rs Server Code Generation Research

**Date:** 2026-02-27
**Purpose:** Evaluate using AWS smithy-rs directly (not s3s) to build an S3-compatible server in Rust via Smithy code generation.

---

## Table of Contents

1. [Overview](#1-overview)
2. [How smithy-rs Server Codegen Works](#2-how-smithy-rs-server-codegen-works)
3. [Generated Code Architecture](#3-generated-code-architecture)
4. [S3 Smithy Model Availability](#4-s3-smithy-model-availability)
5. [Practical Steps to Generate S3 Server Code](#5-practical-steps-to-generate-s3-server-code)
6. [Maturity Assessment](#6-maturity-assessment)
7. [Comparison: smithy-rs Server vs s3s](#7-comparison-smithy-rs-server-vs-s3s)
8. [Known Issues with S3 Server Codegen](#8-known-issues-with-s3-server-codegen)
9. [Conclusion and Recommendation](#9-conclusion-and-recommendation)

---

## 1. Overview

### What is smithy-rs?

[smithy-rs](https://github.com/smithy-lang/smithy-rs) is the official AWS code generator that produces Rust code from [Smithy](https://smithy.io/) interface definition language (IDL) models. It generates:

- **Client SDKs** (the entire `aws-sdk-rust` is generated from it)
- **Server SDKs** (generates server skeletons with routing, serialization, and handler traits)
- **Generic Smithy clients** (non-AWS services)

The code generation is written in **Kotlin** and plugs into the **Smithy Gradle plugin** build system. The generated Rust code depends on runtime crates published under the `aws-smithy-*` namespace.

### Repository

- **GitHub:** https://github.com/smithy-lang/smithy-rs
- **Design docs:** https://smithy-lang.github.io/smithy-rs/design/
- **Server runtime crate:** `aws-smithy-http-server` (v0.66.2, 2026-02-16)
- **Commits:** 3,300+ on main branch
- **Releases:** 173+

---

## 2. How smithy-rs Server Codegen Works

### Code Generation Pipeline

1. You write (or obtain) a **Smithy model** (`.smithy` or `.json` AST format) defining your service, operations, and data types
2. A **Gradle build** invokes the smithy-rs Kotlin code generator via the Smithy Gradle plugin
3. The generator produces a **complete Rust crate** containing:
   - Input/Output/Error structs for each operation
   - Protocol-aware serialization/deserialization (RestJson1, RestXml, AwsJson1.0, AwsJson1.1, RPC v2 CBOR)
   - A service builder that accepts handler functions
   - HTTP routing logic matching requests to operations
   - Tower-compatible service wrappers
4. You write your **business logic** as async handler functions matching the generated operation signatures
5. The handler functions are wired into the generated service builder, producing a hyper/tower HTTP service

### Code Generation Entry Point

The entry point is `RustCodegenPlugin::execute()`, which constructs a `CodegenVisitor` that walks the Smithy model and generates Rust code. The output is a complete Rust crate with `lib.rs`, module structure, and `Cargo.toml`.

### Build System Requirements

- **JDK 17+** (for Gradle and Kotlin codegen)
- **Gradle** (build orchestrator, wrapper included in projects)
- **Rust toolchain** (MSRV: `stable-2`, currently Rust 1.86+)
- **smithy-build.json** (declares code generation targets and settings)
- **build.gradle.kts / settings.gradle.kts** (Gradle configuration)

### Supported Protocols (Server-Side)

| Protocol | Marker Type | S3 Uses? |
|----------|-------------|----------|
| `@restJson1` | `RestJson1` | No |
| `@restXml` | `RestXml` | **Yes** |
| `@awsJson1_0` | `AwsJson10` | No |
| `@awsJson1_1` | `AwsJson11` | No |
| `@rpcv2Cbor` | `RpcV2Cbor` | No |

S3 uses the `@aws.protocols#restXml` protocol with `noErrorWrapping: true`.

---

## 3. Generated Code Architecture

### Operation Shape Pattern

For each Smithy operation, the generator produces a zero-sized type (ZST) marker struct implementing `OperationShape`:

```rust
pub struct GetPokemonSpecies;

impl OperationShape for GetPokemonSpecies {
    const ID: ShapeId = /* ... */;
    type Input = GetPokemonSpeciesInput;
    type Output = GetPokemonSpeciesOutput;
    type Error = GetPokemonSpeciesError;
}
```

### Handler Signature

You implement operations as simple async functions:

```rust
async fn get_pokemon_species(
    input: GetPokemonSpeciesInput,
) -> Result<GetPokemonSpeciesOutput, GetPokemonSpeciesError> {
    // Business logic here
    todo!()
}
```

Handlers can also accept **unmodelled data** via extractors (similar to axum):

```rust
async fn handler(
    input: GetPokemonSpeciesInput,
    Extension(db): Extension<Database>,
) -> Result<GetPokemonSpeciesOutput, GetPokemonSpeciesError> {
    // Access shared state via Extension
}
```

### Service Builder

The generated service builder accepts handler functions per operation:

```rust
let config = PokemonServiceConfig::builder().build();
let app = PokemonService::builder(config)
    .get_pokemon_species(get_pokemon_species_handler)
    .get_storage(get_storage_handler)
    // ... more operations
    .build()
    .expect("failed to build service");
```

Each operation has three setter methods:
- `.operation_name(handler)` -- for async function handlers
- `.operation_name_service(service)` -- for Tower `Service` implementations
- `.operation_name_custom(svc)` -- for pre-built HTTP services

The builder can produce the service via:
- `.build()` -- fails if any operation is missing
- `.build_unchecked()` -- allows missing operations (returns 500 for unimplemented ones)

### Routing

Routing is protocol-aware. The `Router` trait matches HTTP requests to operation-specific services:

```rust
pub trait Router<B> {
    type Service;
    type Error;
    fn match_route(&self, request: &http::Request<B>) -> Result<Self::Service, Self::Error>;
}
```

For `RestXml` (S3's protocol), routing uses HTTP method + URI path + query parameters, matching against the Smithy model's `@http` trait bindings.

### Serialization/Deserialization

Protocol-aware conversion uses two traits:
- `FromRequest<Protocol, B>` -- deserializes HTTP request into operation input
- `IntoResponse<Protocol>` -- serializes operation output/error into HTTP response

The `Upgrade<Protocol, Op, S>` wrapper handles the full cycle: deserialize request, call handler, serialize response.

### Plugin/Middleware System

smithy-rs provides a plugin system built on Tower:
- **HTTP plugins** -- operate on raw HTTP request/response
- **Model plugins** -- operate on typed operation input/output with operation awareness
- Plugins can wrap individual operations or the entire service
- Tower layers (tracing, auth, rate limiting) can be applied

### Running the Server

```rust
let app = PokemonService::builder(config)
    .get_pokemon_species(handler)
    .build()
    .unwrap();

// Convert to hyper service and serve
let addr = SocketAddr::from(([0, 0, 0, 0], 13734));
let server = hyper::Server::bind(&addr).serve(app.into_make_service());
server.await?;
```

---

## 4. S3 Smithy Model Availability

### Official AWS API Models Repository

As of June 2025, AWS publicly released all their service API models:

- **Repository:** https://github.com/aws/api-models-aws
- **Format:** Smithy JSON AST (`.json` files)
- **Organization:** `models/<service-id>/<version>/<service-id>-<version>.json`
- **S3 location:** `models/s3/<version>/s3-<version>.json`
- **Updates:** Published daily
- **Also available on Maven Central**

### S3 Model in smithy-rs

The smithy-rs repository itself contains AWS service models used for SDK generation:
- Located at `aws/sdk/aws-models/s3.json`
- This is the same model used to generate `aws-sdk-s3`

### S3-Specific Smithy Traits

S3 has several custom Smithy traits that affect code generation:
- `@s3UnwrappedXmlOutput` -- Response bodies are not wrapped in the standard RestXml operation-level XML node
- Virtual hosting / path-style addressing configuration
- Dual-stack and transfer acceleration endpoint customizations
- `noErrorWrapping: true` on the RestXml protocol

### S3 Model Complexity

The S3 Smithy model is one of the most complex AWS service models, with:
- 96+ operations
- Extensive use of HTTP bindings (headers, query params, path params)
- Streaming request/response bodies
- Multipart upload flows
- XML serialization with S3-specific quirks
- Presigned URL support requirements
- Event streams (SelectObjectContent)

---

## 5. Practical Steps to Generate S3 Server Code

### Option A: Using the smithy-rs Repository Directly

```bash
# 1. Clone smithy-rs
git clone https://github.com/smithy-lang/smithy-rs.git
cd smithy-rs

# 2. Ensure JDK 17+ is installed

# 3. Generate S3 server SDK (the codegen-server-test already has S3)
#    The codegen-server-test/build.gradle.kts already includes S3 as a test model
./gradlew :codegen-server-test:assemble

# 4. Find the generated server crate in the build output
```

### Option B: Standalone Project (Based on Pokemon Service Template)

**Step 1: Project structure**

```
my-s3-server/
  build.gradle.kts
  settings.gradle.kts
  gradle.properties
  model/
    s3.smithy  (or s3.json from aws/api-models-aws)
  my-s3-service/
    Cargo.toml
    src/
      main.rs  (your business logic)
  smithy-rs/   (git submodule)
```

**Step 2: settings.gradle.kts** -- reference the smithy-rs submodule codegen projects

**Step 3: build.gradle.kts** -- configure the server codegen plugin:

```kotlin
plugins {
    java
    alias(libs.plugins.smithy.gradle.base)
    alias(libs.plugins.smithy.gradle.jar)
}

dependencies {
    implementation(project(":codegen-server"))
    // ... smithy model dependencies
}
```

**Step 4: smithy-build.json** -- declare server generation target:

```json
{
    "version": "1.0",
    "projections": {
        "my-s3-server": {
            "plugins": {
                "rust-server-codegen": {
                    "service": "com.amazonaws.s3#AmazonS3",
                    "module": "s3-server-sdk",
                    "moduleVersion": "0.1.0"
                }
            }
        }
    }
}
```

**Step 5: Generate**

```bash
./gradlew assemble
```

This produces a Rust crate with all S3 operation types, routing, and a builder.

**Step 6: Implement handlers** and wire into the generated builder.

### Key Codegen Settings

From the `codegen-server-test` build configuration, available settings include:
- `debugMode: true/false` -- enables debug output in generated code
- `publicConstrainedTypes: true/false` -- whether constrained types are public
- `ignoreUnsupportedConstraints: true/false` -- skip unsupported Smithy constraints
- `alwaysSendEventStreamInitialResponse: true/false` -- for event stream compatibility

---

## 6. Maturity Assessment

### Stability

**The smithy-rs README explicitly states: "All internal and external interfaces are considered unstable and subject to change without notice."**

This applies to both client and server code generation. The server SDK has never been declared "generally available" or "production ready" by AWS.

### Evidence of Active Development

| Indicator | Status |
|-----------|--------|
| Commits on main | 3,300+ |
| Total releases | 173+ |
| `aws-smithy-http-server` latest | v0.66.2 (2026-02-16) |
| Recent server features | Event stream initial-response, RPC v2 CBOR, custom validation |
| MSRV | Rust 1.88.0 |
| CI | Active, main branch CI runs regularly |

### Who Uses It?

- **AWS internal services** reportedly use smithy-rs server codegen (the Pokemon service example was created by an AWS employee)
- **No known public S3-compatible servers** built with smithy-rs server SDK
- The server SDK is primarily used for **non-S3** services (RestJson1, AwsJson protocols)
- S3 is included in `codegen-server-test` as a **test model**, not as a production reference

### What Protocols Are Mature?

| Protocol | Server Maturity |
|----------|----------------|
| RestJson1 | Most mature, well-tested |
| AwsJson1.0/1.1 | Well-tested |
| RPC v2 CBOR | Recently added, growing |
| RestXml (S3) | **Test coverage exists but S3-specific challenges remain** |

---

## 7. Comparison: smithy-rs Server vs s3s

### Architecture

| Aspect | smithy-rs Server | s3s |
|--------|-----------------|-----|
| **Approach** | Full Smithy codegen (Kotlin/Gradle produces Rust crate) | Custom Rust codegen (reads Smithy model, generates Rust with Python/Rust tooling) |
| **Build system** | Gradle + JDK 17+ + smithy-rs submodule | Pure Cargo (codegen is a Rust build step) |
| **Generated output** | Complete Rust crate with operation types, routing, builder | Rust trait with 96 methods + DTO types + HTTP adapter |
| **Handler pattern** | One async fn per operation, wired into builder | One trait with 96 async methods, implement what you need |
| **Protocol handling** | Generic (supports multiple Smithy protocols) | S3-specific (hardcoded for S3 REST protocol) |
| **HTTP framework** | Tower/hyper | hyper + tower compatible |
| **Auth** | Plugin/middleware based, you implement | Built-in SigV4/V2 verification, you provide secret key lookup |
| **Runtime crate** | `aws-smithy-http-server` (0.66.2) | `s3s` (0.12.0 stable, 0.13.0-alpha.3) |

### Developer Experience

| Aspect | smithy-rs Server | s3s |
|--------|-----------------|-----|
| **Setup complexity** | High (JDK, Gradle, submodule, codegen step) | Low (add `s3s` to Cargo.toml) |
| **Incremental builds** | Slow (Gradle codegen + Cargo build) | Fast (standard Cargo) |
| **IDE support** | Excellent after codegen (pure Rust) | Excellent (pure Rust crate) |
| **Learning curve** | Steep (Smithy concepts, Tower plugins, codegen config) | Moderate (implement trait methods) |
| **Model updates** | Re-run Gradle codegen | Update s3s crate version |

### S3 Specifics

| Aspect | smithy-rs Server | s3s |
|--------|-----------------|-----|
| **S3 operation coverage** | All operations generated from model | All 96 operations in trait |
| **S3 XML quirks** | Must handle S3-specific XML customizations | Already handled (purpose-built for S3) |
| **Virtual hosting** | Not built-in, must implement | Built-in `S3Host` trait with `SingleDomain`/`MultiDomain` |
| **SigV4 auth** | Not built-in for server | Built-in verification |
| **Presigned URLs** | Not built-in for server | Built-in support |
| **S3 error format** | Generic Smithy errors | S3-specific XML error responses with proper codes |
| **S3 checksums** | Basic from model | Full support (CRC32, CRC32C, CRC64NVME, SHA256, MD5) |
| **Streaming** | Smithy streaming blob types | S3-specific `ByteStream` + `StreamingBlob` |

### Tradeoffs Summary

**smithy-rs Server advantages:**
- Authoritative: generated from the same model AWS uses
- Generic: works for any Smithy service, not just S3
- Tower ecosystem: full access to Tower middleware
- Type safety: compile-time checks that all operations are handled
- Future-proof: model updates automatically flow to generated code

**smithy-rs Server disadvantages:**
- Heavy build toolchain (JDK + Gradle + Git submodule)
- S3-specific features (auth, virtual hosting, presigned URLs) not included
- RestXml server codegen has known issues with S3 model
- Unstable API, subject to breaking changes
- No production examples for S3 specifically
- You must implement S3-specific protocol quirks yourself

**s3s advantages:**
- Purpose-built for S3
- Pure Rust, no JVM toolchain needed
- Built-in SigV4/V2 auth, virtual hosting, presigned URLs
- S3-specific error handling and XML serialization
- Well-documented reference implementation (s3s-fs)
- Active community with 260+ stars, 32 releases
- Default implementations return `NotImplemented` -- implement incrementally

**s3s disadvantages:**
- S3-only (not generic for other Smithy services)
- Experimental status, no security protections built-in
- Uses `async_trait` for object safety (minor overhead)
- Fewer Tower integration patterns

---

## 8. Known Issues with S3 Server Codegen

### Type Mismatch in Required Payload Members

When generating server code from the S3 model, there are compilation errors where the `?` operator produces incompatible types. Specifically, required members marked with `@httpPayload` traits cause deserialization code to expect `Option<T>` when the server struct uses `T` directly.

**Example error:**
```
error[E0308]: `?` operator has incompatible types
  --> delete_objects.rs
   |   crate::http_serde::deser_payload_delete_objects_delete_objects_input_delete(&bytes)?
   |   expected `Option<Delete>`, found struct `Delete`
```

This is tracked in [Issue #1159](https://github.com/smithy-lang/smithy-rs/issues/1159).

### Client/Server Struct Incompatibility

Server-generated structs have different field types than client-generated structs:
- **Client structs:** All fields are `Option<T>` (builder pattern)
- **Server structs:** Only truly optional fields are `Option<T>`

This means you cannot trivially convert between client and server types, and `transmute` fails due to size differences.

### S3 XML Customizations

S3's `@s3UnwrappedXmlOutput` trait and `noErrorWrapping` require special handling that the generic RestXml server codegen may not fully account for.

### Manual Hacks Required

The s3s project explicitly notes that it "applies manual hacks to fix some problems in smithy server codegen." This suggests that raw smithy-rs server output for S3 needs post-processing to be usable.

---

## 9. Conclusion and Recommendation

### Assessment

Using smithy-rs server codegen directly for an S3-compatible server is **technically possible but not practical** at this time for the following reasons:

1. **Build complexity:** The JVM/Gradle toolchain adds significant build infrastructure complexity
2. **S3 codegen issues:** Known compilation errors when generating from the S3 model (Issue #1159), requiring manual fixes
3. **Missing S3 features:** No built-in SigV4 auth verification, virtual hosting, presigned URLs, or S3-specific error formatting on the server side
4. **Unstable API:** All interfaces explicitly declared unstable
5. **No precedent:** No known S3-compatible servers built with smithy-rs server SDK in production
6. **Maintenance burden:** Must maintain JDK + Gradle + smithy-rs submodule in addition to Rust code

### Recommendation

**Use s3s for building the S3-compatible server.** The s3s crate:
- Is purpose-built for exactly this use case
- Provides everything needed out of the box (auth, routing, virtual hosting, error handling)
- Has a pure Rust build chain
- Has a well-documented reference implementation (s3s-fs)
- Is actively maintained with regular releases
- Already handles the S3-specific Smithy codegen issues internally

smithy-rs server codegen would be the right choice if you were building a **non-S3 service** (e.g., a custom REST API defined in Smithy) or if AWS eventually stabilizes the server SDK and resolves the S3-specific codegen issues. For an S3-compatible server specifically, s3s is the clear winner.

---

## Sources

- [smithy-rs GitHub Repository](https://github.com/smithy-lang/smithy-rs) -- Main project repository
- [The Anatomy of a Service (smithy-rs design docs)](https://smithy-lang.github.io/smithy-rs/design/server/anatomy.html) -- Server architecture documentation
- [Generating Common Service Code (smithy-rs design docs)](https://smithy-lang.github.io/smithy-rs/design/server/code_generation.html) -- Code generation internals
- [aws-smithy-http-server on docs.rs](https://docs.rs/crate/aws-smithy-http-server/latest) -- Server runtime crate documentation
- [aws/api-models-aws](https://github.com/aws/api-models-aws) -- Official AWS service Smithy models
- [AWS Blog: Introducing AWS API models](https://aws.amazon.com/blogs/aws/introducing-aws-api-models-and-publicly-available-resources-for-aws-api-definitions/) -- AWS API models announcement
- [Amazon S3 Customizations (Smithy spec)](https://smithy.io/2.0/aws/customizations/s3-customizations.html) -- S3-specific Smithy traits
- [AWS restXml protocol (Smithy spec)](https://smithy.io/2.0/aws/protocols/aws-restxml-protocol.html) -- Protocol S3 uses
- [Issue #1159: Server deserialize with required member](https://github.com/smithy-lang/smithy-rs/issues/1159) -- S3 server codegen type mismatch bug
- [crisidev/smithy-rs-pokemon-service](https://github.com/crisidev/smithy-rs-pokemon-service) -- Standalone example project
- [timClicks/smithy-rs-demo](https://github.com/timClicks/smithy-rs-demo) -- Demo/template project
- [smithy-rs examples directory](https://github.com/awslabs/smithy-rs/tree/main/examples) -- Official examples (pokemon-service, TLS, Lambda)
- [s3s GitHub Repository](https://github.com/Nugine/s3s) -- S3 Service Adapter
- [s3s on crates.io](https://crates.io/crates/s3s) -- s3s crate
- [Smithy Gradle Plugins](https://smithy.io/2.0/guides/gradle-plugin/index.html) -- Build system documentation
- [Creating a Codegen Repo (Smithy docs)](https://smithy.io/2.0/guides/building-codegen/creating-codegen-repo.html) -- How to set up a codegen project
- [InfoQ: AWS Open-Sources Smithy API Models](https://www.infoq.com/news/2025/06/aws-smithy-api-models-opensource/) -- Coverage of AWS model release
