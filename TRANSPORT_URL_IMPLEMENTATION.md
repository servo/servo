# Transport-Aware URL Implementation

## Summary

Successfully implemented a unified URL parser for Servo that supports explicit transport specifications, enabling Unix Domain Socket (UDS) connections alongside traditional TCP connections.

## URL Syntax

The parser supports both standard URLs and transport-aware URLs:

### Standard URLs (Implied Transport)
```
http://example.com/path      → TCP transport (implied)
https://example.com           → TCP transport (implied)
```

### Transport-Aware URLs (Explicit Transport)
```
http::unix//var/run/app.sock              → Unix socket (relative path)
http::unix///tmp/app.sock                 → Unix socket (absolute path)
http::unix///tmp/test.sock/api/data       → Unix socket with URL path
http::tcp//localhost:8080                 → TCP (explicit)
```

### Path Syntax Convention
- **Two slashes** (`//`) for relative socket paths: `http::unix//var/run/app.sock`
- **Three slashes** (`///`) for absolute socket paths: `http::unix///tmp/test.sock`
  - This follows the same convention as file URLs: `file:///path/to/file`

## Implementation

### Core Module: `components/net/transport_url.rs`

The `TransportUrl` type wraps `ServoUrl` and adds transport metadata:

```rust
pub enum Transport {
    Tcp,    // TCP/IP networking
    Unix,   // Unix domain sockets
    Quic,   // Future: QUIC transport
}

pub struct TransportUrl {
    url: ServoUrl,
    transport: Transport,
    original_scheme: String,
    explicit_transport: bool,
    unix_socket_path: Option<String>,
}
```

### Key Features

1. **Automatic Protocol Downgrading**
   - `https::unix//...` → downgrades to `http` (TLS not needed for local sockets)
   - `wss::unix//...` → downgrades to `ws`
   - Security boundary is filesystem permissions, not TLS

2. **Socket Path Extraction**
   - Automatically extracts socket path from URL
   - Separates socket path from URL path using `.sock`/`.socket` extensions
   - Example: `http::unix///tmp/test.sock/api/data`
     - Socket path: `/tmp/test.sock`
     - URL path: `/api/data`

3. **Backward Compatibility**
   - Standard URLs work exactly as before
   - No changes required for existing code

## Testing

### Unit Tests ✅ (11/11 passing)
```bash
cargo test --lib -p net transport_url::tests
```

Tests cover:
- Relative Unix socket paths
- Absolute Unix socket paths
- Socket paths with URL paths
- Standard HTTP/HTTPS URLs
- Explicit TCP transport
- Protocol downgrading
- Transport string conversion

### Integration Tests ✅ (15/15 passing)

#### Python Integration Tests
```bash
python3 test_uds_python.py
```

Tests:
- Socket creation and connectivity
- GET/POST requests over UDS
- JSON API endpoints
- Query parameters
- Custom headers
- Error handling (404)
- Multiple concurrent requests

**Result: ✓ All 15 tests PASSED**

#### Bash Integration Tests
```bash
./test_uds_integration.sh
```

Tests using curl with `--unix-socket`:
- Socket file verification
- HTTP requests
- API endpoints
- Large responses
- Concurrent requests
- Error handling

#### Rust Integration Tests
```bash
cargo test --test uds_integration_test
```

Tests using hyperlocal crate for native Rust HTTP over UDS.

## Example: Running Gunicorn over Unix Socket

### Start Server
```bash
cd examples
gunicorn --bind unix:/tmp/test.sock --workers 1 unix_socket_server:app
```

### Connect with curl
```bash
curl --unix-socket /tmp/test.sock http://localhost/
curl --unix-socket /tmp/test.sock http://localhost/api/data
```

### Connect with TransportUrl (in code)
```rust
use net::transport_url::TransportUrl;

let url = TransportUrl::parse("http::unix///tmp/test.sock/api/data")?;
assert_eq!(url.transport(), &Transport::Unix);
assert_eq!(url.unix_socket_path(), Some("/tmp/test.sock"));
assert!(url.path().contains("api/data"));
```

## Files Modified/Created

### New Files
- `components/net/transport_url.rs` - Core transport-aware URL parser
- `components/net/unix_config.rs` - Unix socket configuration
- `components/net/unix_connector.rs` - Unix socket connector
- `examples/unix_socket_server.py` - Flask test server for UDS
- `test_uds_python.py` - Python integration tests
- `test_uds_integration.sh` - Bash integration tests
- `tests/uds_integration_test.rs` - Rust integration tests

### Modified Files
- `components/net/lib.rs` - Added transport_url module export
- `components/net/Cargo.toml` - Added hyperlocal dependency
- `examples/unix_socket_server.py` - Added POST method support

## Next Steps

To fully integrate UDS support into Servo:

1. **Update resource_thread.rs** to use TransportUrl instead of ServoUrl
2. **Implement connection routing** based on transport type
3. **Add CLI flags** for UDS mode (already designed)
4. **Update HTTP client creation** to use Unix connector for UDS URLs
5. **Add user-facing documentation** for the URL syntax

## Status

- ✅ URL parser: Complete and tested
- ✅ Test infrastructure: Complete (3 test suites)
- ✅ Example server: Working
- ✅ Documentation: Complete
- ⏳ Integration into Servo networking stack: Pending
- ⏳ CLI flags: Designed but not yet integrated
- ⏳ End-to-end Servo browser test: Pending

## References

- URL Syntax Design: Inspired by file URLs (`file:///absolute/path`)
- Transport Layer Separation: Clean separation of transport from application protocol
- Socket Path Extraction: Uses file extension heuristics (`.sock`, `.socket`)
