# Documentation

## Research

- [LocalStack S3 Implementation Research](./research/localstack-s3-research.md) - Comprehensive analysis of what a Rust-based LocalStack S3 service needs to support for feature parity.
- [s3s Crate Ecosystem Research](./research/s3s-crate-research.md) - Comprehensive analysis of the s3s Rust crate for building S3-compatible services.
- [LocalStack Container & CI Research](./research/localstack-container-ci-research.md) - How LocalStack's Docker container works, gateway routing through port 4566, health endpoints, and the official GitHub Action for CI.
- [S3 Integration Test Suites Research](./research/s3-integration-test-suites-research.md) - Survey of available S3 compliance/integration test suites (Ceph s3-tests, MinIO Mint, MSST-S3, s3s-e2e) and recommendations for adoption.
- [smithy-rs Server Codegen Research](./research/smithy-rs-server-codegen-research.md) - Evaluation of using AWS smithy-rs directly to build an S3-compatible server via Smithy code generation, compared with s3s.
- [DynamoDB API Research](./research/dynamodb-api-research.md) - Comprehensive analysis of the AWS DynamoDB API surface, protocol, data model, Smithy model, and what a Rust-based local DynamoDB implementation would need.
