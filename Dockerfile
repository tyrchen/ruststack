# syntax=docker/dockerfile:1

# ---------------------------------------------------------------------------
# Builder stage: compile a fully static binary with musl
# ---------------------------------------------------------------------------
FROM rust:1.94-slim AS builder

RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*

WORKDIR /src

# Copy the toolchain file first so rustup installs the correct version,
# then add musl targets to the resolved toolchain.
COPY rust-toolchain.toml ./
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

# Copy manifests to cache dependency compilation.
COPY Cargo.toml Cargo.lock ./
COPY apps/ruststack-server/Cargo.toml apps/ruststack-server/Cargo.toml
COPY crates/ruststack-auth/Cargo.toml crates/ruststack-auth/Cargo.toml
COPY crates/ruststack-core/Cargo.toml crates/ruststack-core/Cargo.toml
COPY crates/ruststack-s3-core/Cargo.toml crates/ruststack-s3-core/Cargo.toml
COPY crates/ruststack-s3-http/Cargo.toml crates/ruststack-s3-http/Cargo.toml
COPY crates/ruststack-s3-model/Cargo.toml crates/ruststack-s3-model/Cargo.toml
COPY crates/ruststack-s3-xml/Cargo.toml crates/ruststack-s3-xml/Cargo.toml
COPY crates/ruststack-dynamodb-model/Cargo.toml crates/ruststack-dynamodb-model/Cargo.toml
COPY crates/ruststack-dynamodb-http/Cargo.toml crates/ruststack-dynamodb-http/Cargo.toml
COPY crates/ruststack-dynamodb-core/Cargo.toml crates/ruststack-dynamodb-core/Cargo.toml
COPY crates/ruststack-dynamodbstreams-model/Cargo.toml crates/ruststack-dynamodbstreams-model/Cargo.toml
COPY crates/ruststack-dynamodbstreams-http/Cargo.toml crates/ruststack-dynamodbstreams-http/Cargo.toml
COPY crates/ruststack-dynamodbstreams-core/Cargo.toml crates/ruststack-dynamodbstreams-core/Cargo.toml
COPY crates/ruststack-sqs-model/Cargo.toml crates/ruststack-sqs-model/Cargo.toml
COPY crates/ruststack-sqs-http/Cargo.toml crates/ruststack-sqs-http/Cargo.toml
COPY crates/ruststack-sqs-core/Cargo.toml crates/ruststack-sqs-core/Cargo.toml
COPY crates/ruststack-ssm-model/Cargo.toml crates/ruststack-ssm-model/Cargo.toml
COPY crates/ruststack-ssm-http/Cargo.toml crates/ruststack-ssm-http/Cargo.toml
COPY crates/ruststack-ssm-core/Cargo.toml crates/ruststack-ssm-core/Cargo.toml
COPY crates/ruststack-sns-model/Cargo.toml crates/ruststack-sns-model/Cargo.toml
COPY crates/ruststack-sns-http/Cargo.toml crates/ruststack-sns-http/Cargo.toml
COPY crates/ruststack-sns-core/Cargo.toml crates/ruststack-sns-core/Cargo.toml
COPY crates/ruststack-lambda-model/Cargo.toml crates/ruststack-lambda-model/Cargo.toml
COPY crates/ruststack-lambda-http/Cargo.toml crates/ruststack-lambda-http/Cargo.toml
COPY crates/ruststack-lambda-core/Cargo.toml crates/ruststack-lambda-core/Cargo.toml
COPY crates/ruststack-events-model/Cargo.toml crates/ruststack-events-model/Cargo.toml
COPY crates/ruststack-events-http/Cargo.toml crates/ruststack-events-http/Cargo.toml
COPY crates/ruststack-events-core/Cargo.toml crates/ruststack-events-core/Cargo.toml
COPY crates/ruststack-logs-model/Cargo.toml crates/ruststack-logs-model/Cargo.toml
COPY crates/ruststack-logs-http/Cargo.toml crates/ruststack-logs-http/Cargo.toml
COPY crates/ruststack-logs-core/Cargo.toml crates/ruststack-logs-core/Cargo.toml
COPY crates/ruststack-kms-model/Cargo.toml crates/ruststack-kms-model/Cargo.toml
COPY crates/ruststack-kms-http/Cargo.toml crates/ruststack-kms-http/Cargo.toml
COPY crates/ruststack-kms-core/Cargo.toml crates/ruststack-kms-core/Cargo.toml
COPY crates/ruststack-kinesis-model/Cargo.toml crates/ruststack-kinesis-model/Cargo.toml
COPY crates/ruststack-kinesis-http/Cargo.toml crates/ruststack-kinesis-http/Cargo.toml
COPY crates/ruststack-kinesis-core/Cargo.toml crates/ruststack-kinesis-core/Cargo.toml
COPY crates/ruststack-secretsmanager-model/Cargo.toml crates/ruststack-secretsmanager-model/Cargo.toml
COPY crates/ruststack-secretsmanager-http/Cargo.toml crates/ruststack-secretsmanager-http/Cargo.toml
COPY crates/ruststack-secretsmanager-core/Cargo.toml crates/ruststack-secretsmanager-core/Cargo.toml
COPY crates/ruststack-ses-model/Cargo.toml crates/ruststack-ses-model/Cargo.toml
COPY crates/ruststack-ses-http/Cargo.toml crates/ruststack-ses-http/Cargo.toml
COPY crates/ruststack-ses-core/Cargo.toml crates/ruststack-ses-core/Cargo.toml
COPY crates/ruststack-apigatewayv2-model/Cargo.toml crates/ruststack-apigatewayv2-model/Cargo.toml
COPY crates/ruststack-apigatewayv2-http/Cargo.toml crates/ruststack-apigatewayv2-http/Cargo.toml
COPY crates/ruststack-apigatewayv2-core/Cargo.toml crates/ruststack-apigatewayv2-core/Cargo.toml
COPY crates/ruststack-cloudwatch-model/Cargo.toml crates/ruststack-cloudwatch-model/Cargo.toml
COPY crates/ruststack-cloudwatch-http/Cargo.toml crates/ruststack-cloudwatch-http/Cargo.toml
COPY crates/ruststack-cloudwatch-core/Cargo.toml crates/ruststack-cloudwatch-core/Cargo.toml
COPY crates/ruststack-iam-model/Cargo.toml crates/ruststack-iam-model/Cargo.toml
COPY crates/ruststack-iam-http/Cargo.toml crates/ruststack-iam-http/Cargo.toml
COPY crates/ruststack-iam-core/Cargo.toml crates/ruststack-iam-core/Cargo.toml
COPY crates/ruststack-sts-model/Cargo.toml crates/ruststack-sts-model/Cargo.toml
COPY crates/ruststack-sts-http/Cargo.toml crates/ruststack-sts-http/Cargo.toml
COPY crates/ruststack-sts-core/Cargo.toml crates/ruststack-sts-core/Cargo.toml
COPY tests/integration/Cargo.toml tests/integration/Cargo.toml

# Create stub sources so cargo can resolve the workspace.
RUN mkdir -p apps/ruststack-server/src && echo 'fn main() {}' > apps/ruststack-server/src/main.rs \
    && mkdir -p crates/ruststack-auth/src && echo '//! stub' > crates/ruststack-auth/src/lib.rs \
    && mkdir -p crates/ruststack-core/src && echo '//! stub' > crates/ruststack-core/src/lib.rs \
    && mkdir -p crates/ruststack-s3-core/src && echo '//! stub' > crates/ruststack-s3-core/src/lib.rs \
    && mkdir -p crates/ruststack-s3-http/src && echo '//! stub' > crates/ruststack-s3-http/src/lib.rs \
    && mkdir -p crates/ruststack-s3-model/src && echo '//! stub' > crates/ruststack-s3-model/src/lib.rs \
    && mkdir -p crates/ruststack-s3-xml/src && echo '//! stub' > crates/ruststack-s3-xml/src/lib.rs \
    && mkdir -p crates/ruststack-dynamodb-model/src && echo '//! stub' > crates/ruststack-dynamodb-model/src/lib.rs \
    && mkdir -p crates/ruststack-dynamodb-http/src && echo '//! stub' > crates/ruststack-dynamodb-http/src/lib.rs \
    && mkdir -p crates/ruststack-dynamodb-core/src && echo '//! stub' > crates/ruststack-dynamodb-core/src/lib.rs \
    && mkdir -p crates/ruststack-dynamodbstreams-model/src && echo '//! stub' > crates/ruststack-dynamodbstreams-model/src/lib.rs \
    && mkdir -p crates/ruststack-dynamodbstreams-http/src && echo '//! stub' > crates/ruststack-dynamodbstreams-http/src/lib.rs \
    && mkdir -p crates/ruststack-dynamodbstreams-core/src && echo '//! stub' > crates/ruststack-dynamodbstreams-core/src/lib.rs \
    && mkdir -p crates/ruststack-sqs-model/src && echo '//! stub' > crates/ruststack-sqs-model/src/lib.rs \
    && mkdir -p crates/ruststack-sqs-http/src && echo '//! stub' > crates/ruststack-sqs-http/src/lib.rs \
    && mkdir -p crates/ruststack-sqs-core/src && echo '//! stub' > crates/ruststack-sqs-core/src/lib.rs \
    && mkdir -p crates/ruststack-ssm-model/src && echo '//! stub' > crates/ruststack-ssm-model/src/lib.rs \
    && mkdir -p crates/ruststack-ssm-http/src && echo '//! stub' > crates/ruststack-ssm-http/src/lib.rs \
    && mkdir -p crates/ruststack-ssm-core/src && echo '//! stub' > crates/ruststack-ssm-core/src/lib.rs \
    && mkdir -p crates/ruststack-sns-model/src && echo '//! stub' > crates/ruststack-sns-model/src/lib.rs \
    && mkdir -p crates/ruststack-sns-http/src && echo '//! stub' > crates/ruststack-sns-http/src/lib.rs \
    && mkdir -p crates/ruststack-sns-core/src && echo '//! stub' > crates/ruststack-sns-core/src/lib.rs \
    && mkdir -p crates/ruststack-lambda-model/src && echo '//! stub' > crates/ruststack-lambda-model/src/lib.rs \
    && mkdir -p crates/ruststack-lambda-http/src && echo '//! stub' > crates/ruststack-lambda-http/src/lib.rs \
    && mkdir -p crates/ruststack-lambda-core/src && echo '//! stub' > crates/ruststack-lambda-core/src/lib.rs \
    && mkdir -p crates/ruststack-events-model/src && echo '//! stub' > crates/ruststack-events-model/src/lib.rs \
    && mkdir -p crates/ruststack-events-http/src && echo '//! stub' > crates/ruststack-events-http/src/lib.rs \
    && mkdir -p crates/ruststack-events-core/src && echo '//! stub' > crates/ruststack-events-core/src/lib.rs \
    && mkdir -p crates/ruststack-logs-model/src && echo '//! stub' > crates/ruststack-logs-model/src/lib.rs \
    && mkdir -p crates/ruststack-logs-http/src && echo '//! stub' > crates/ruststack-logs-http/src/lib.rs \
    && mkdir -p crates/ruststack-logs-core/src && echo '//! stub' > crates/ruststack-logs-core/src/lib.rs \
    && mkdir -p crates/ruststack-kms-model/src && echo '//! stub' > crates/ruststack-kms-model/src/lib.rs \
    && mkdir -p crates/ruststack-kms-http/src && echo '//! stub' > crates/ruststack-kms-http/src/lib.rs \
    && mkdir -p crates/ruststack-kms-core/src && echo '//! stub' > crates/ruststack-kms-core/src/lib.rs \
    && mkdir -p crates/ruststack-kinesis-model/src && echo '//! stub' > crates/ruststack-kinesis-model/src/lib.rs \
    && mkdir -p crates/ruststack-kinesis-http/src && echo '//! stub' > crates/ruststack-kinesis-http/src/lib.rs \
    && mkdir -p crates/ruststack-kinesis-core/src && echo '//! stub' > crates/ruststack-kinesis-core/src/lib.rs \
    && mkdir -p crates/ruststack-secretsmanager-model/src && echo '//! stub' > crates/ruststack-secretsmanager-model/src/lib.rs \
    && mkdir -p crates/ruststack-secretsmanager-http/src && echo '//! stub' > crates/ruststack-secretsmanager-http/src/lib.rs \
    && mkdir -p crates/ruststack-secretsmanager-core/src && echo '//! stub' > crates/ruststack-secretsmanager-core/src/lib.rs \
    && mkdir -p crates/ruststack-ses-model/src && echo '//! stub' > crates/ruststack-ses-model/src/lib.rs \
    && mkdir -p crates/ruststack-ses-http/src && echo '//! stub' > crates/ruststack-ses-http/src/lib.rs \
    && mkdir -p crates/ruststack-ses-core/src && echo '//! stub' > crates/ruststack-ses-core/src/lib.rs \
    && mkdir -p crates/ruststack-apigatewayv2-model/src && echo '//! stub' > crates/ruststack-apigatewayv2-model/src/lib.rs \
    && mkdir -p crates/ruststack-apigatewayv2-http/src && echo '//! stub' > crates/ruststack-apigatewayv2-http/src/lib.rs \
    && mkdir -p crates/ruststack-apigatewayv2-core/src && echo '//! stub' > crates/ruststack-apigatewayv2-core/src/lib.rs \
    && mkdir -p crates/ruststack-cloudwatch-model/src && echo '//! stub' > crates/ruststack-cloudwatch-model/src/lib.rs \
    && mkdir -p crates/ruststack-cloudwatch-http/src && echo '//! stub' > crates/ruststack-cloudwatch-http/src/lib.rs \
    && mkdir -p crates/ruststack-cloudwatch-core/src && echo '//! stub' > crates/ruststack-cloudwatch-core/src/lib.rs \
    && mkdir -p crates/ruststack-iam-model/src && echo '//! stub' > crates/ruststack-iam-model/src/lib.rs \
    && mkdir -p crates/ruststack-iam-http/src && echo '//! stub' > crates/ruststack-iam-http/src/lib.rs \
    && mkdir -p crates/ruststack-iam-core/src && echo '//! stub' > crates/ruststack-iam-core/src/lib.rs \
    && mkdir -p crates/ruststack-sts-model/src && echo '//! stub' > crates/ruststack-sts-model/src/lib.rs \
    && mkdir -p crates/ruststack-sts-http/src && echo '//! stub' > crates/ruststack-sts-http/src/lib.rs \
    && mkdir -p crates/ruststack-sts-core/src && echo '//! stub' > crates/ruststack-sts-core/src/lib.rs \
    && mkdir -p tests/integration/src && echo '//! stub' > tests/integration/src/lib.rs

# Pre-build dependencies (cached layer).
ARG TARGETARCH
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    if [ "$TARGETARCH" = "arm64" ]; then \
      RUST_TARGET=aarch64-unknown-linux-musl; \
    else \
      RUST_TARGET=x86_64-unknown-linux-musl; \
    fi && \
    cargo build --release --target "$RUST_TARGET" -p ruststack-server 2>/dev/null || true

# Copy real source code.
COPY crates/ crates/
COPY apps/ apps/
COPY tests/ tests/

# Touch source files so cargo knows they changed.
RUN find crates/ apps/ tests/ -name '*.rs' -exec touch {} +

# Build the actual binary.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    if [ "$TARGETARCH" = "arm64" ]; then \
      RUST_TARGET=aarch64-unknown-linux-musl; \
    else \
      RUST_TARGET=x86_64-unknown-linux-musl; \
    fi && \
    cargo build --release --target "$RUST_TARGET" -p ruststack-server && \
    cp "/src/target/$RUST_TARGET/release/ruststack-server" /ruststack-server

# ---------------------------------------------------------------------------
# Runtime stage: scratch image with just the binary
# ---------------------------------------------------------------------------
FROM scratch

COPY --from=builder /ruststack-server /ruststack-server
COPY --from=builder /tmp /tmp

ENV GATEWAY_LISTEN=0.0.0.0:4566
ENV LOG_LEVEL=info
ENV SERVICES=

EXPOSE 4566

HEALTHCHECK --interval=2s --timeout=3s --start-period=1s --retries=3 \
    CMD ["/ruststack-server", "--health-check"]

ENTRYPOINT ["/ruststack-server"]
