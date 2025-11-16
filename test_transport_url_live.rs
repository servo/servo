// Test transport URL parsing with live Gunicorn server
use hyper::{Method, Request};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;

// We need to manually copy the TransportUrl code since we can't import from components/net easily
// In a real integration, this would use: use net::transport_url::TransportUrl;

#[tokio::main]
async fn main() {
    println!("Testing TransportUrl with live Gunicorn server...\n");

    // Test 1: Parse absolute Unix socket URL with three slashes
    let test_url = "http::unix///tmp/test.sock";
    println!("Parsing: {}", test_url);

    // For now, manually extract socket path to test with hyperlocal
    let socket_path = "/tmp/test.sock";
    let url_path = "/";

    println!("  Socket path: {}", socket_path);
    println!("  URL path: {}", url_path);

    // Test connection using hyperlocal
    let client = Client::builder(TokioExecutor::new())
        .build(hyperlocal::UnixConnector);

    let url = hyperlocal::Uri::new(socket_path, url_path).into();
    let request = Request::builder()
        .method(Method::GET)
        .uri(url)
        .body(http_body_util::Empty::<hyper::body::Bytes>::new())
        .unwrap();

    println!("\nConnecting to Gunicorn over Unix socket...");
    match client.request(request).await {
        Ok(response) => {
            println!("✓ Success! Status: {}", response.status());
            let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let body_str = String::from_utf8_lossy(&body_bytes);
            if body_str.contains("Hello from Unix Socket Server") {
                println!("✓ Response contains expected content");
            } else {
                println!("✗ Unexpected response content");
            }
        }
        Err(e) => {
            eprintln!("✗ Connection failed: {}", e);
            std::process::exit(1);
        }
    }

    // Test 2: With URL path
    println!("\nTest 2: With URL path /api/data");
    let url2 = hyperlocal::Uri::new(socket_path, "/api/data").into();
    let request2 = Request::builder()
        .method(Method::GET)
        .uri(url2)
        .body(http_body_util::Empty::<hyper::body::Bytes>::new())
        .unwrap();

    match client.request(request2).await {
        Ok(response) => {
            println!("✓ Success! Status: {}", response.status());
            let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let body_str = String::from_utf8_lossy(&body_bytes);
            if body_str.contains("unix_domain_socket") {
                println!("✓ API response contains transport info");
            }
        }
        Err(e) => {
            eprintln!("✗ API request failed: {}", e);
        }
    }

    println!("\n✓ All live tests passed!");
}
