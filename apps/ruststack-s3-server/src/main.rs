//! RustStack S3 Server - High-performance S3-compatible server.
//!
//! This binary provides a LocalStack-compatible S3 server built on `ruststack-s3-http`.
//! It handles S3 protocol translation, virtual-hosted-style addressing,
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

mod handler;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as HttpConnBuilder;
use ruststack_s3_auth::StaticCredentialProvider;
use tokio::net::TcpListener;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use ruststack_s3_core::{RustStackS3, S3Config};
use ruststack_s3_http::service::{S3HttpConfig, S3HttpService};

use crate::handler::RustStackHandler;

/// Server version reported in health check responses.
const VERSION: &str = env!("CARGO_PKG_VERSION");

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

/// Build the [`S3HttpConfig`] from the application [`S3Config`].
fn build_http_config(config: &S3Config) -> S3HttpConfig {
    // Build a credential provider from environment variables if available.
    let credential_provider = build_credential_provider();

    S3HttpConfig {
        domain: config.s3_domain.clone(),
        virtual_hosting: config.s3_virtual_hosting,
        skip_signature_validation: config.s3_skip_signature_validation,
        region: config.default_region.clone(),
        credential_provider,
    }
}

/// Build a credential provider from `ACCESS_KEY` / `SECRET_KEY` environment
/// variables (used by MinIO Mint and other test harnesses).
fn build_credential_provider() -> Option<Arc<dyn ruststack_s3_auth::CredentialProvider>> {
    let access_key = std::env::var("ACCESS_KEY")
        .or_else(|_| std::env::var("AWS_ACCESS_KEY_ID"))
        .ok()?;
    let secret_key = std::env::var("SECRET_KEY")
        .or_else(|_| std::env::var("AWS_SECRET_ACCESS_KEY"))
        .ok()?;

    info!(
        access_key = %access_key,
        "configured credential provider from environment"
    );

    Some(Arc::new(StaticCredentialProvider::new(vec![(
        access_key, secret_key,
    )])))
}

/// Run the accept loop, serving connections until a shutdown signal is received.
async fn serve<H: ruststack_s3_http::dispatch::S3Handler>(
    listener: TcpListener,
    service: S3HttpService<H>,
) -> Result<()> {
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

    let provider = RustStackS3::new(config.clone());
    let handler = RustStackHandler(provider);
    let http_config = build_http_config(&config);
    let service = S3HttpService::new(handler, http_config);

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
    fn test_should_build_http_config_from_s3_config() {
        let config = S3Config::from_env();
        let http_config = build_http_config(&config);

        assert_eq!(http_config.domain, config.s3_domain);
        assert_eq!(http_config.virtual_hosting, config.s3_virtual_hosting);
        assert_eq!(
            http_config.skip_signature_validation,
            config.s3_skip_signature_validation
        );
        assert_eq!(http_config.region, config.default_region);
    }
}
