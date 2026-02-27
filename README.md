# RustStack

A high-performance, LocalStack-compatible AWS service emulator written in Rust.

Currently implements **S3** with full protocol support including versioning, multipart uploads, CORS, tagging, ACLs, object lock, and more.

## Quick Start

```bash
cargo build -p ruststack-s3-server
S3_SKIP_SIGNATURE_VALIDATION=true ./target/debug/ruststack-s3-server
```

The server listens on `0.0.0.0:4566` by default, compatible with AWS SDK clients pointed at `http://localhost:4566`.

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `GATEWAY_LISTEN` | `0.0.0.0:4566` | Bind address |
| `S3_SKIP_SIGNATURE_VALIDATION` | `true` | Skip SigV4 verification |
| `S3_DOMAIN` | `s3.localhost.localstack.cloud` | Virtual hosting domain |
| `LOG_LEVEL` | `info` | Log level filter |

## Architecture

```
ruststack-s3-model   — S3 types generated from Smithy model
ruststack-s3-xml     — XML serialization/deserialization
ruststack-s3-auth    — SigV4 authentication
ruststack-s3-http    — HTTP routing, request/response conversion
ruststack-s3-core    — Business logic and in-memory storage
ruststack-s3-server  — Server binary
```

## License

This project is distributed under the terms of MIT.

See [LICENSE](LICENSE.md) for details.

Copyright 2025 Tyr Chen
