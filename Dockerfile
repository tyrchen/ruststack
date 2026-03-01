# syntax=docker/dockerfile:1

# ---------------------------------------------------------------------------
# Builder stage: compile a fully static binary with musl
# ---------------------------------------------------------------------------
FROM rust:1.86-slim AS builder

RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*

WORKDIR /src

# Copy the toolchain file first so rustup installs the correct version,
# then add musl targets to the resolved toolchain.
COPY rust-toolchain.toml ./
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

# Copy manifests to cache dependency compilation.
COPY Cargo.toml Cargo.lock ./
COPY apps/ruststack-s3-server/Cargo.toml apps/ruststack-s3-server/Cargo.toml
COPY crates/ruststack-auth/Cargo.toml crates/ruststack-auth/Cargo.toml
COPY crates/ruststack-core/Cargo.toml crates/ruststack-core/Cargo.toml
COPY crates/ruststack-s3-core/Cargo.toml crates/ruststack-s3-core/Cargo.toml
COPY crates/ruststack-s3-http/Cargo.toml crates/ruststack-s3-http/Cargo.toml
COPY crates/ruststack-s3-model/Cargo.toml crates/ruststack-s3-model/Cargo.toml
COPY crates/ruststack-s3-xml/Cargo.toml crates/ruststack-s3-xml/Cargo.toml
COPY tests/integration/Cargo.toml tests/integration/Cargo.toml

# Create stub sources so cargo can resolve the workspace.
RUN mkdir -p apps/ruststack-s3-server/src && echo 'fn main() {}' > apps/ruststack-s3-server/src/main.rs \
    && mkdir -p crates/ruststack-auth/src && echo '//! stub' > crates/ruststack-auth/src/lib.rs \
    && mkdir -p crates/ruststack-core/src && echo '//! stub' > crates/ruststack-core/src/lib.rs \
    && mkdir -p crates/ruststack-s3-core/src && echo '//! stub' > crates/ruststack-s3-core/src/lib.rs \
    && mkdir -p crates/ruststack-s3-http/src && echo '//! stub' > crates/ruststack-s3-http/src/lib.rs \
    && mkdir -p crates/ruststack-s3-model/src && echo '//! stub' > crates/ruststack-s3-model/src/lib.rs \
    && mkdir -p crates/ruststack-s3-xml/src && echo '//! stub' > crates/ruststack-s3-xml/src/lib.rs \
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
    cargo build --release --target "$RUST_TARGET" -p ruststack-s3-server 2>/dev/null || true

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
    cargo build --release --target "$RUST_TARGET" -p ruststack-s3-server && \
    cp "/src/target/$RUST_TARGET/release/ruststack-s3-server" /ruststack-s3-server

# ---------------------------------------------------------------------------
# Runtime stage: scratch image with just the binary
# ---------------------------------------------------------------------------
FROM scratch

COPY --from=builder /ruststack-s3-server /ruststack-s3-server
COPY --from=builder /tmp /tmp

ENV GATEWAY_LISTEN=0.0.0.0:4566
ENV LOG_LEVEL=info
ENV S3_SKIP_SIGNATURE_VALIDATION=true

EXPOSE 4566

HEALTHCHECK --interval=2s --timeout=3s --start-period=1s --retries=3 \
    CMD ["/ruststack-s3-server", "--health-check"]

ENTRYPOINT ["/ruststack-s3-server"]
