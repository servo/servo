// Rust integration tests for Unix Domain Socket connections
//
// Tests using hyperlocal to verify Gunicorn serves correctly over UDS

use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

use hyper::body::Buf;
use hyper::{Method, Request};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use tokio::runtime::Runtime;

const SOCKET_PATH: &str = "/tmp/servo-uds-rust-test/test.sock";

/// Helper to start Gunicorn server
fn start_gunicorn() -> std::io::Result<Child> {
    // Create socket directory
    std::fs::create_dir_all("/tmp/servo-uds-rust-test")?;

    // Remove old socket
    let _ = std::fs::remove_file(SOCKET_PATH);

    // Start Gunicorn
    let child = Command::new("gunicorn")
        .args(&[
            "--bind",
            &format!("unix:{}", SOCKET_PATH),
            "--workers",
            "1",
            "--log-level",
            "warning",
            "unix_socket_server:app",
        ])
        .current_dir("examples")
        .spawn()?;

    // Wait for socket to be created
    for _ in 0..20 {
        if std::path::Path::new(SOCKET_PATH).exists() {
            thread::sleep(Duration::from_millis(500)); // Extra time for server ready
            return Ok(child);
        }
        thread::sleep(Duration::from_millis(500));
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "Socket was not created in time",
    ))
}

/// Helper to stop Gunicorn
fn stop_gunicorn(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_file(SOCKET_PATH);
}

#[test]
fn test_uds_connection_basic() {
    let mut server = start_gunicorn().expect("Failed to start Gunicorn");

    let rt = Runtime::new().unwrap();
    let result = rt.block_on(async {
        // Create hyperlocal client
        let client = Client::builder(TokioExecutor::new())
            .build(hyperlocal::UnixConnector);

        // Create request
        let url = hyperlocal::Uri::new(SOCKET_PATH, "/").into();
        let request = Request::builder()
            .method(Method::GET)
            .uri(url)
            .body(http_body_util::Empty::<hyper::body::Bytes>::new())
            .unwrap();

        // Send request
        let response = client.request(request).await.unwrap();

        // Check status
        assert_eq!(response.status(), 200);

        // Read body
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);

        assert!(body_str.contains("Hello from Unix Socket Server"));
    });

    stop_gunicorn(server);
    result
}

#[test]
fn test_uds_api_endpoint() {
    let mut server = start_gunicorn().expect("Failed to start Gunicorn");

    let rt = Runtime::new().unwrap();
    let result = rt.block_on(async {
        let client = Client::builder(TokioExecutor::new())
            .build(hyperlocal::UnixConnector);

        let url = hyperlocal::Uri::new(SOCKET_PATH, "/api/data").into();
        let request = Request::builder()
            .method(Method::GET)
            .uri(url)
            .body(http_body_util::Empty::<hyper::body::Bytes>::new())
            .unwrap();

        let response = client.request(request).await.unwrap();
        assert_eq!(response.status(), 200);

        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);

        assert!(body_str.contains("unix_domain_socket"));
        assert!(body_str.contains("gunicorn"));
    });

    stop_gunicorn(server);
    result
}

#[test]
fn test_uds_404_error() {
    let mut server = start_gunicorn().expect("Failed to start Gunicorn");

    let rt = Runtime::new().unwrap();
    let result = rt.block_on(async {
        let client = Client::builder(TokioExecutor::new())
            .build(hyperlocal::UnixConnector);

        let url = hyperlocal::Uri::new(SOCKET_PATH, "/nonexistent").into();
        let request = Request::builder()
            .method(Method::GET)
            .uri(url)
            .body(http_body_util::Empty::<hyper::body::Bytes>::new())
            .unwrap();

        let response = client.request(request).await.unwrap();
        assert_eq!(response.status(), 404);
    });

    stop_gunicorn(server);
    result
}

#[test]
fn test_uds_custom_headers() {
    let mut server = start_gunicorn().expect("Failed to start Gunicorn");

    let rt = Runtime::new().unwrap();
    let result = rt.block_on(async {
        let client = Client::builder(TokioExecutor::new())
            .build(hyperlocal::UnixConnector);

        let url = hyperlocal::Uri::new(SOCKET_PATH, "/api/data").into();
        let request = Request::builder()
            .method(Method::GET)
            .uri(url)
            .header("X-Custom-Header", "test-value")
            .header("User-Agent", "Servo-Test/1.0")
            .body(http_body_util::Empty::<hyper::body::Bytes>::new())
            .unwrap();

        let response = client.request(request).await.unwrap();
        assert_eq!(response.status(), 200);

        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);

        // The API endpoint returns headers in the response
        assert!(body_str.contains("X-Custom-Header"));
        assert!(body_str.contains("test-value"));
    });

    stop_gunicorn(server);
    result
}

#[test]
fn test_uds_multiple_requests() {
    let mut server = start_gunicorn().expect("Failed to start Gunicorn");

    let rt = Runtime::new().unwrap();
    let result = rt.block_on(async {
        let client = Client::builder(TokioExecutor::new())
            .build(hyperlocal::UnixConnector);

        // Make 5 requests
        for i in 0..5 {
            let url = hyperlocal::Uri::new(SOCKET_PATH, "/").into();
            let request = Request::builder()
                .method(Method::GET)
                .uri(url)
                .body(http_body_util::Empty::<hyper::body::Bytes>::new())
                .unwrap();

            let response = client.request(request).await.unwrap();
            assert_eq!(
                response.status(),
                200,
                "Request {} failed",
                i + 1
            );
        }
    });

    stop_gunicorn(server);
    result
}
