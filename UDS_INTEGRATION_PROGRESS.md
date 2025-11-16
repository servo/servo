# Unix Socket Integration Progress

## Current Status: Infrastructure Complete ✅

The Unix Domain Socket (UDS) integration for Servo has reached a key milestone. The core infrastructure is in place, with the transport-aware URL parser fully tested and the HTTP client architecture updated to support runtime routing between TCP and Unix sockets.

## Completed Work

### 1. Transport-Aware URL Parser ✅
**File:** `components/net/transport_url.rs`

- **Syntax Support:**
  - Standard URLs: `http://example.com` (implied TCP)
  - Unix sockets (relative): `http::unix//var/run/app.sock`
  - Unix sockets (absolute): `http::unix///tmp/test.sock`
  - Explicit TCP: `http::tcp//localhost:8080`

- **Features:**
  - Automatic protocol downgrading (HTTPS→HTTP for local transports)
  - Socket path extraction using file extension heuristics
  - Backward compatible with standard URLs
  - Support for both 2-slash and 3-slash path conventions

- **Test Coverage:**
  - ✅ 11/11 unit tests passing
  - ✅ 15/15 Python integration tests passing
  - ✅ Bash integration tests with curl
  - ✅ Rust integration tests with hyperlocal

### 2. HTTP Client Infrastructure ✅
**Files:** `components/net/http_loader.rs`, `components/net/connector.rs`, `components/net/resource_thread.rs`

- **HttpState Updates:**
  ```rust
  pub struct HttpState {
      pub client: Client<Connector, BoxedBody>,              // TCP client
      pub unix_client: Option<Client<UnixConnector, BoxedBody>>, // Unix client
      pub socket_mapping: Option<SocketMapping>,             // Path mappings
      // ... other fields
  }
  ```

- **Client Creation:**
  - `create_http_client()` - Creates TCP/TLS client
  - `create_unix_http_client()` - Creates Unix socket client
  - Unix client initialized when `SERVO_USE_UNIX_SOCKETS=1`

- **Type System Solution:**
  - Instead of unifying Future types at compile time (impossible due to type differences)
  - Maintain separate clients and route at runtime based on URL transport
  - Clean separation avoids type system conflicts

### 3. Configuration System ✅
**Files:** `components/net/unix_config.rs`, `components/net/unix_connector.rs`

- Environment variable configuration:
  - `SERVO_USE_UNIX_SOCKETS=1` - Enable UDS mode
  - `SERVO_SOCKET_DIR=/path` - Default socket directory
  - `SERVO_SOCKET_MAPPINGS=host1:/path1,host2:/path2` - Host mappings

- CLI flags designed (in `ports/servoshell/prefs.rs`):
  - `--unix-sockets` - Enable UDS
  - `--socket-dir=<path>` - Socket directory
  - `--socket-mapping=<map>` - Hostname mappings

### 4. Test Infrastructure ✅
**Files:** `test_uds_python.py`, `test_uds_integration.sh`, `tests/uds_integration_test.rs`

- **Python Tests (15 tests):**
  - Socket connectivity
  - GET/POST requests over UDS
  - JSON API endpoints
  - Query parameters & headers
  - Error handling (404)
  - Concurrent requests

- **Bash Tests:**
  - curl with `--unix-socket` flag
  - HTTP request verification
  - API endpoint testing

- **Rust Tests:**
  - Native hyperlocal integration
  - Async request handling

### 5. Example Server ✅
**File:** `examples/unix_socket_server.py`

- Flask application serving over Unix sockets
- Multiple routes: `/`, `/api/data`, `/test`, `/about`
- POST method support
- Run with: `gunicorn --bind unix:/tmp/test.sock unix_socket_server:app`

## Remaining Work

### 1. Request Routing Logic ⏳
**File:** `components/net/http_loader.rs` (function: `http_network_fetch`)

**What's Needed:**
```rust
// In http_network_fetch(), around line 1997-2073:

// 1. Parse URL to detect transport
let transport_url = TransportUrl::parse(url.as_str())?;

// 2. Route based on transport type
let (response, msg) = if transport_url.transport() == &Transport::Unix {
    // Unix socket path
    if let (Some(unix_client), Some(mapping)) =
        (&context.state.unix_client, &context.state.socket_mapping)
    {
        let socket_path = mapping.get_socket_path_for_url(&url)?;
        let unix_uri = hyperlocal::Uri::new(&socket_path, url.path()).into();

        // Build request with Unix URI
        let request = HyperRequest::builder()
            .method(&request.method)
            .uri(unix_uri)
            .body(body)?;

        // Use Unix client
        unix_client.request(request).await?
    } else {
        return Response::network_error(NetworkError::Internal("Unix sockets not configured"));
    }
} else {
    // TCP path (existing code)
    obtain_response(&context.state.client, ...)
};
```

**Challenges:**
- Need to adapt `obtain_response` logic for Unix sockets
- Handle URI construction with hyperlocal
- Maintain devtools integration
- Preserve timing/metrics collection

### 2. URL Mapping Logic ⏳
**File:** `components/net/unix_connector.rs`

**What's Needed:**
- Implement `SocketMapping::get_socket_path_for_url()`
- Handle both explicit socket paths and hostname mappings
- Support TransportUrl socket path extraction

**Example:**
```rust
impl SocketMapping {
    pub fn get_socket_path_for_url(&self, url: &ServoUrl) -> Result<PathBuf, Error> {
        // Check if URL has explicit socket path (from TransportUrl)
        if let Some(path) = transport_url.unix_socket_path() {
            return Ok(PathBuf::from(path));
        }

        // Check hostname mappings
        if let Some(host) = url.host_str() {
            if let Some(path) = self.get_mapping(host) {
                return Ok(path);
            }
        }

        // Fallback to default directory + hostname.sock
        Ok(self.default_socket_dir.join(format!("{}.sock", host)))
    }
}
```

### 3. Integration Testing 🔍
**What's Needed:**
- End-to-end test with Servo browser
- Headless mode test: `servo -z http::unix///tmp/test.sock -o screenshot.png`
- Verify screenshot is generated correctly
- Test with actual Gunicorn server

**Test Script:**
```bash
#!/bin/bash
# Start Gunicorn
cd examples
gunicorn --bind unix:/tmp/test.sock --workers 1 --daemon unix_socket_server:app

# Wait for socket
sleep 2

# Test with Servo (headless)
servo -z -o /tmp/servo-test.png "http::unix///tmp/test.sock"

# Verify screenshot exists and has content
if [ -f /tmp/servo-test.png ] && [ -s /tmp/servo-test.png ]; then
    echo "✓ Success: Screenshot generated"
else
    echo "✗ Failed: No screenshot"
    exit 1
fi

# Cleanup
killall gunicorn
```

### 4. CLI Flag Integration 🔧
**File:** `ports/servoshell/prefs.rs`

**Status:** Flags are defined but need environment variable setting verified

**What's Needed:**
- Verify environment variables are set correctly from CLI flags
- Test flag combinations:
  - `--unix-sockets --socket-dir=/tmp/sockets`
  - `--socket-mapping=example.com:/tmp/example.sock`
- Documentation for users

## Architecture Decisions

### Why Runtime Routing?

The initial approach attempted to unify TCP and Unix socket connectors at the type level:

```rust
pub enum ConnectorType {
    Tcp(HyperHttpConnector),
    Unix(ServoUnixConnector),
}
```

**Problem:** Different Future types
- `HyperHttpConnector::call()` returns `HttpConnecting<GaiResolver>`
- `UnixConnector::call()` returns `Pin<Box<dyn Future>>`
- Cannot be unified in a single enum variant

**Solution:** Runtime routing with separate clients
- HttpState holds both clients
- Parse URL at request time
- Route to appropriate client based on transport
- No type-level unification needed

### Benefits of This Approach

1. **Type Safety:** Each client maintains its native type
2. **Performance:** No boxing/dynamic dispatch for TCP (common case)
3. **Flexibility:** Easy to add more transports (QUIC, etc.)
4. **Testing:** Each transport can be tested independently
5. **Maintainability:** Clear separation of concerns

## File Summary

### New Files (Created)
- `components/net/transport_url.rs` (363 lines) - URL parser
- `components/net/unix_config.rs` (93 lines) - Configuration
- `components/net/unix_connector.rs` (114 lines) - Unix connector
- `examples/unix_socket_server.py` (150+ lines) - Test server
- `test_uds_python.py` (300+ lines) - Python integration tests
- `test_uds_integration.sh` (400+ lines) - Bash integration tests
- `tests/uds_integration_test.rs` (150+ lines) - Rust integration tests
- `TRANSPORT_URL_IMPLEMENTATION.md` - Comprehensive documentation

### Modified Files
- `components/net/lib.rs` - Module exports
- `components/net/Cargo.toml` - Added hyperlocal dependency
- `components/net/connector.rs` - Added `create_unix_http_client()`
- `components/net/http_loader.rs` - Added Unix client fields to HttpState
- `components/net/resource_thread.rs` - Unix client initialization
- `ports/servoshell/prefs.rs` - CLI flags (existing)

## Next Steps Priority

1. **[HIGH] Implement request routing in http_network_fetch** (est. 2-3 hours)
   - Parse URLs with TransportUrl
   - Route to appropriate client
   - Handle Unix URI construction

2. **[MEDIUM] Add URL mapping logic** (est. 1 hour)
   - Implement `get_socket_path_for_url()`
   - Support explicit paths and mappings

3. **[MEDIUM] End-to-end testing** (est. 1-2 hours)
   - Test with real Gunicorn server
   - Verify headless screenshots
   - Document any issues

4. **[LOW] CLI integration verification** (est. 30 min)
   - Test flag combinations
   - Verify environment variables

5. **[LOW] Documentation** (est. 1 hour)
   - User-facing docs
   - Example usage
   - Troubleshooting guide

## Estimated Time to Completion

- **Core functionality:** 3-4 hours
- **Testing & verification:** 2-3 hours
- **Documentation & polish:** 1-2 hours
- **Total:** 6-9 hours of focused development

## Test Commands

```bash
# Unit tests
cargo test --lib -p net transport_url::tests

# Python integration tests
python3 test_uds_python.py

# Bash integration tests
./test_uds_integration.sh

# Build check
cargo check -p net

# Full Servo build (slow)
./mach build -d
```

## Success Criteria

- ✅ URL parser: 11/11 tests passing
- ✅ Infrastructure: Unix client created and initialized
- ⏳ Request routing: Pending implementation
- ⏳ End-to-end: Servo loads page over Unix socket
- ⏳ CLI: Flags work correctly
- ⏳ Documentation: User guide complete

## Conclusion

The infrastructure for Unix socket support in Servo is complete and tested. The transport-aware URL parser successfully handles both standard and Unix socket URLs, with comprehensive test coverage. The HTTP client architecture has been updated to support runtime routing between transports, solving the type system challenges.

The remaining work focuses on connecting these pieces together:  implementing the request routing logic in `http_network_fetch()`, adding URL-to-socket-path mapping, and end-to-end testing. The architecture is sound and the path forward is clear.
