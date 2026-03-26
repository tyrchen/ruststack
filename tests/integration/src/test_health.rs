//! Health check integration tests.
//!
//! Verifies that the `/_localstack/health` endpoint works correctly and that
//! raw TCP health checks (as used by Docker `HEALTHCHECK`) do not produce
//! "connection closed before message completed" errors.

#[cfg(test)]
mod tests {
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    };

    use crate::endpoint_url;

    /// Extract host:port from the endpoint URL.
    fn endpoint_addr() -> String {
        let url = endpoint_url();
        url.strip_prefix("http://").unwrap_or(&url).to_owned()
    }

    /// Simulate the exact same TCP-level health check that the Docker
    /// HEALTHCHECK uses (`ruststack-server --health-check`).  The fix for
    /// issue #1 removed the `writer.shutdown()` call; this test verifies
    /// the server responds correctly without a TCP half-close.
    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_respond_to_raw_tcp_health_check() {
        let addr = endpoint_addr();

        let mut stream = TcpStream::connect(&addr)
            .await
            .unwrap_or_else(|e| panic!("cannot connect to {addr}: {e}"));

        let request = format!(
            "GET /_localstack/health HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n",
        );
        stream
            .write_all(request.as_bytes())
            .await
            .expect("failed to write request");
        // Intentionally NOT calling stream.shutdown() — this is the fix.

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .await
            .expect("failed to read response");

        assert!(
            response.contains("200 OK"),
            "expected 200 OK in response, got: {response}",
        );
        assert!(
            response.contains("\"running\""),
            "expected 'running' in response, got: {response}",
        );
    }

    /// Run multiple sequential health checks to ensure the server does not
    /// accumulate connection errors over time.
    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_should_handle_repeated_health_checks() {
        let addr = endpoint_addr();

        for i in 0..10 {
            let mut stream = TcpStream::connect(&addr)
                .await
                .unwrap_or_else(|e| panic!("connect #{i} to {addr} failed: {e}"));

            let request = format!(
                "GET /_localstack/health HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n",
            );
            stream
                .write_all(request.as_bytes())
                .await
                .unwrap_or_else(|e| panic!("write #{i} failed: {e}"));

            let mut response = String::new();
            stream
                .read_to_string(&mut response)
                .await
                .unwrap_or_else(|e| panic!("read #{i} failed: {e}"));

            assert!(
                response.contains("200 OK"),
                "health check #{i} failed: {response}",
            );
        }
    }
}
