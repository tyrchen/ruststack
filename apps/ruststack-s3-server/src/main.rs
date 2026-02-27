//! RustStack S3 Server - High-performance S3-compatible server.
//!
//! This binary provides a LocalStack-compatible S3 server built on the `s3s` crate.
//! It handles S3 protocol translation, authentication, virtual-hosted-style addressing,
//! and exposes health check endpoints for orchestration systems.
//!
//! # Usage
//!
//! ```text
//! GATEWAY_LISTEN=0.0.0.0:4566 ruststack-s3-server
//! ```
//!
//! # Environment Variables
//!
//! | Variable | Default | Description |
//! |----------|---------|-------------|
//! | `GATEWAY_LISTEN` | `0.0.0.0:4566` | Bind address |
//! | `S3_SKIP_SIGNATURE_VALIDATION` | `true` | Skip SigV4 verification |
//! | `S3_DOMAIN` | `s3.localhost.localstack.cloud` | Virtual hosting domain |
//! | `LOG_LEVEL` | `info` | Log level filter |
//! | `RUST_LOG` | *(unset)* | Fine-grained tracing filter (overrides `LOG_LEVEL`) |

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::{Context, Result};
use bytes::Bytes;
use hyper::body::Incoming;
use hyper::service::Service;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as HttpConnBuilder;
use s3s::service::S3ServiceBuilder;
use tokio::net::TcpListener;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use ruststack_s3_core::auth::RustStackAuth;
use ruststack_s3_core::{RustStackS3, S3Config};

/// Server version reported in health check responses.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// JSON health check response body.
fn health_response_body() -> Bytes {
    // Pre-format the JSON. This is a small, static response so allocation is fine.
    Bytes::from(format!(
        r#"{{"services":{{"s3":"running"}},"edition":"ruststack","version":"{VERSION}"}}"#,
    ))
}

/// A wrapper service that intercepts health check paths before delegating to s3s.
///
/// Routes `/_localstack/health` and `/health` to a JSON health response.
/// All other requests are forwarded to the inner `SharedS3Service`.
#[derive(Debug, Clone)]
struct HealthCheckService {
    inner: s3s::service::SharedS3Service,
    health_body: Arc<Bytes>,
}

impl HealthCheckService {
    fn new(inner: s3s::service::SharedS3Service) -> Self {
        Self {
            inner,
            health_body: Arc::new(health_response_body()),
        }
    }
}

impl Service<Request<Incoming>> for HealthCheckService {
    type Response = Response<s3s::Body>;
    type Error = s3s::S3Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let path = req.uri().path();

        if path == "/_localstack/health" || path == "/health" {
            let body = self.health_body.clone();
            return Box::pin(async move {
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(s3s::Body::from((*body).clone()))
                    .map_err(|e| {
                        s3s::S3Error::with_source(
                            s3s::S3ErrorCode::InternalError,
                            Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                        )
                    })?;
                Ok(response)
            });
        }

        // Delegate to the inner s3s service.
        self.inner.call(req)
    }
}

/// Initialize the tracing subscriber.
///
/// Uses `RUST_LOG` if set, otherwise falls back to the `LOG_LEVEL` config value.
fn init_tracing(log_level: &str) -> Result<()> {
    let filter = if std::env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else {
        EnvFilter::try_new(log_level)
            .with_context(|| format!("invalid log level filter: {log_level}"))?
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .init();

    Ok(())
}

/// Build the s3s service from configuration.
fn build_s3_service(config: &S3Config) -> Result<s3s::service::SharedS3Service> {
    let provider = RustStackS3::new(config.clone());
    let auth = RustStackAuth::new(config.s3_skip_signature_validation);

    let mut builder = S3ServiceBuilder::new(provider);
    builder.set_auth(auth);

    if config.s3_virtual_hosting {
        let host = s3s::host::SingleDomain::new(&config.s3_domain)
            .with_context(|| format!("invalid S3 domain: {}", config.s3_domain))?;
        builder.set_host(host);
    }

    Ok(builder.build().into_shared())
}

/// Run the accept loop, serving connections until a shutdown signal is received.
async fn serve(listener: TcpListener, service: HealthCheckService) -> Result<()> {
    let graceful = hyper_util::server::graceful::GracefulShutdown::new();
    let http = HttpConnBuilder::new(TokioExecutor::new());

    let shutdown = async {
        tokio::signal::ctrl_c().await.ok();
        info!("received shutdown signal, draining connections");
    };

    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, peer_addr) = match result {
                    Ok(conn) => conn,
                    Err(e) => {
                        warn!(error = %e, "failed to accept connection");
                        continue;
                    }
                };

                let svc = service.clone();
                let conn = http.serve_connection(TokioIo::new(stream), svc);
                let conn = graceful.watch(conn.into_owned());

                tokio::spawn(async move {
                    if let Err(e) = conn.await {
                        error!(peer_addr = %peer_addr, error = %e, "connection error");
                    }
                });
            }

            () = &mut shutdown => {
                info!("shutting down gracefully");
                break;
            }
        }
    }

    // Wait for in-flight requests to complete.
    graceful.shutdown().await;
    info!("all connections drained, exiting");

    Ok(())
}

/// Perform a health check by connecting to the gateway and requesting the health endpoint.
///
/// Exits with code 0 if healthy, 1 otherwise.
async fn run_health_check(addr: &str) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    let stream = TcpStream::connect(addr)
        .await
        .with_context(|| format!("cannot connect to {addr}"))?;

    let (mut reader, mut writer) = stream.into_split();

    let request =
        format!("GET /_localstack/health HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    writer.write_all(request.as_bytes()).await?;
    writer.shutdown().await?;

    let mut response = String::new();
    reader.read_to_string(&mut response).await?;

    if response.contains("200 OK") && response.contains("\"s3\":\"running\"") {
        Ok(())
    } else {
        anyhow::bail!("unhealthy response from {addr}")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Handle --health-check flag for Docker HEALTHCHECK.
    if std::env::args().any(|a| a == "--health-check") {
        let config = S3Config::from_env();
        let addr = config.gateway_listen.replace("0.0.0.0", "127.0.0.1");
        let healthy = run_health_check(&addr).await.is_ok();
        std::process::exit(i32::from(!healthy));
    }

    let config = S3Config::from_env();

    init_tracing(&config.log_level)?;

    info!(
        gateway_listen = %config.gateway_listen,
        s3_domain = %config.s3_domain,
        s3_virtual_hosting = config.s3_virtual_hosting,
        s3_skip_signature_validation = config.s3_skip_signature_validation,
        version = VERSION,
        "starting RustStack S3 Server",
    );

    let s3_service = build_s3_service(&config)?;
    let service = HealthCheckService::new(s3_service);

    let addr: SocketAddr = config
        .gateway_listen
        .parse()
        .with_context(|| format!("invalid bind address: {}", config.gateway_listen))?;

    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind to {addr}"))?;

    info!(%addr, "listening for connections");

    serve(listener, service).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_produce_valid_health_json() {
        let body = health_response_body();
        let value: serde_json::Value =
            serde_json::from_slice(&body).expect("health body should be valid JSON");

        assert_eq!(value["services"]["s3"], "running");
        assert_eq!(value["edition"], "ruststack");
        assert_eq!(value["version"], VERSION);
    }
}
