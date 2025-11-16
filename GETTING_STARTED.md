# Getting Started with Servo UDS

This guide will help you set up, test, and run Servo with Unix Domain Socket support from scratch.

## Quick Start (Ubuntu 25.10)

```bash
# 1. Setup development environment
./setup_dev_environment.sh

# 2. Run all tests
./run_all_tests.sh

# 3. Launch demo
./launch_demo.sh

# 4. Verify no TCP connections
./verify_no_tcp.sh
```

## Detailed Guide

### Prerequisites

- Ubuntu 25.10 (or similar Debian-based distribution)
- Internet connection for package downloads
- ~10 GB free disk space (for Servo build)
- sudo access for installing packages

### Step 1: Environment Setup

The setup script will install all required dependencies:

```bash
./setup_dev_environment.sh
```

**What it installs:**
- Servo build dependencies (compilers, libraries, headers)
- Rust toolchain (stable)
- Python virtual environment with Flask and Gunicorn
- Network monitoring tools (ss, lsof, netstat)
- Git hooks for code quality checks

**Duration:** ~5-10 minutes (depending on your internet speed)

**After setup:**
```bash
# Activate Python virtual environment
source venv/bin/activate

# Verify installation
rustc --version
python --version
```

### Step 2: Build Servo (First Time)

```bash
# Build in debug mode (faster compilation, slower runtime)
./mach build -d

# OR build in release mode (slower compilation, faster runtime)
./mach build --release
```

**Duration:** 30-60 minutes for first build
**Note:** Subsequent builds are much faster (incremental compilation)

### Step 3: Run Tests

The comprehensive test runner executes all test suites:

```bash
./run_all_tests.sh
```

**Test suites included:**
1. **Rust Unit Tests** (transport_url, unix_socket)
   - 11 tests for URL parsing
   - 13 tests for socket configuration

2. **Python Integration Tests**
   - 15 tests with Gunicorn over Unix sockets
   - GET/POST requests, API endpoints, error handling

3. **Bash Integration Tests**
   - curl-based tests
   - Socket creation, HTTP requests

4. **Rust Integration Tests**
   - hyperlocal integration
   - Native async request handling

5. **TCP Verification** ⚠️ **CRITICAL**
   - Verifies NO TCP listeners
   - Verifies NO TCP connections
   - Ensures UDS-only operation

6. **Compilation Check**
   - cargo check for errors
   - Warning detection

7. **Code Formatting**
   - Rust formatting check

**Expected output:**
```
========================================
Test Summary
========================================
Total tests run: XX
Passed: XX
Failed: 0
Skipped: 0

Success rate: 100%

========================================
✓ ALL TESTS PASSED!
========================================
```

**Logs:** Check `/tmp/servo-uds-tests-TIMESTAMP.log` for details

### Step 4: Verify No TCP Connections

**This is the most important test** - it verifies that the system operates in UDS-only mode with NO TCP connections, not even loopback.

```bash
./verify_no_tcp.sh
```

**What it checks:**
- ✅ No TCP listeners on standard ports (80, 443, 8000, 8080, etc.)
- ✅ No TCP listeners from Gunicorn process
- ✅ No TCP connections from Gunicorn process
- ✅ No loopback (127.0.0.1) TCP connections
- ✅ Unix socket exists and is active
- ✅ No TCP activity during HTTP requests
- ✅ No TCP file descriptors in Gunicorn process

**Expected output:**
```
========================================
VERIFICATION SUMMARY
========================================
Total tests: 7
Passed: 7
Failed: 0

========================================
✓ TCP VERIFICATION PASSED!
========================================

No TCP connections detected.
System is operating in UDS-only mode.
```

**Report file:** `/tmp/servo-tcp-verification-TIMESTAMP.txt`

### Step 5: Launch Demo

The demo launcher provides an interactive demonstration:

```bash
./launch_demo.sh
```

**What it does:**
1. Starts Gunicorn on Unix socket (`/tmp/servo-demo.sock`)
2. Verifies no TCP connections
3. Tests server with curl
4. Offers to launch Servo browser

**Demo modes:**
- **Headless** - Generate screenshot (no GUI required)
- **GUI** - Interactive browser window
- **Server-only** - Just run the server for manual testing

**For server-only mode:**
```bash
./launch_demo.sh --server-only
```

**Manual testing while server runs:**
```bash
# Test with curl
curl --unix-socket /tmp/servo-demo.sock http://localhost/
curl --unix-socket /tmp/servo-demo.sock http://localhost/api/data

# Test with Servo
export SERVO_USE_UNIX_SOCKETS=1
export SERVO_SOCKET_MAPPINGS="localhost:/tmp/servo-demo.sock"
./target/debug/servo "http::unix///tmp/servo-demo.sock"
```

## URL Syntax

### Standard URLs (TCP - backward compatible)
```
http://example.com
https://example.com
```

### Unix Socket URLs (UDS - new syntax)

**Relative paths (2 slashes):**
```
http::unix//var/run/app.sock
http::unix//relative/path/server.sock
```

**Absolute paths (3 slashes):**
```
http::unix///tmp/server.sock
http::unix///var/run/docker.sock
```

**With URL paths:**
```
http::unix///tmp/server.sock/api/data
http::unix///tmp/server.sock/index.html?query=value
```

**Explicit TCP (optional):**
```
http::tcp//localhost:8080
http::tcp//example.com:80
```

## Environment Variables

Configure Unix socket mode via environment variables:

```bash
# Enable Unix socket mode
export SERVO_USE_UNIX_SOCKETS=1

# Default socket directory
export SERVO_SOCKET_DIR=/tmp/sockets

# Hostname-to-socket mappings
export SERVO_SOCKET_MAPPINGS="localhost:/tmp/local.sock,api.local:/tmp/api.sock"

# Enable debug logging
export RUST_LOG=net=debug
```

## CLI Flags (Planned)

```bash
# Enable Unix sockets
servo --unix-sockets http://localhost

# Specify socket directory
servo --unix-sockets --socket-dir=/tmp/sockets http://localhost

# Add socket mapping
servo --socket-mapping=example.com:/tmp/example.sock http://example.com
```

## Troubleshooting

### Build Fails

**Issue:** Build errors with libudev-sys or similar
**Solution:**
```bash
sudo apt-get install libudev-dev pkg-config
./mach clean
./mach build -d
```

### Tests Fail

**Issue:** TCP verification fails
**Solution:**
```bash
# Check what's listening
ss -tlnp | grep gunicorn

# Kill any TCP-listening Gunicorn instances
pkill -f "gunicorn.*--bind 0.0.0.0"
pkill -f "gunicorn.*--bind 127.0.0.1"

# Start fresh
./verify_no_tcp.sh
```

**Issue:** Python tests fail with "Module not found"
**Solution:**
```bash
source venv/bin/activate
pip install flask gunicorn requests requests-unixsocket
```

### Demo Fails

**Issue:** "Socket not created"
**Solution:**
```bash
# Check if port 8000 is in use
ss -tlnp | grep 8000

# Remove old socket files
rm -f /tmp/*.sock /tmp/servo-*.sock

# Check Gunicorn logs
tail -f /tmp/gunicorn-*.log
```

**Issue:** Servo crashes or fails to load
**Solution:**
```bash
# Check environment variables
echo $SERVO_USE_UNIX_SOCKETS
echo $SERVO_SOCKET_MAPPINGS

# Run with debug logging
RUST_LOG=net=debug ./target/debug/servo "http::unix///tmp/server.sock"
```

### Permission Issues

**Issue:** "Permission denied" accessing socket
**Solution:**
```bash
# Check socket permissions
ls -la /tmp/*.sock

# Fix permissions
chmod 666 /tmp/server.sock
```

## Directory Structure

```
servoipc/
├── setup_dev_environment.sh    # Environment setup
├── run_all_tests.sh            # Comprehensive test runner
├── launch_demo.sh              # Automated demo launcher
├── verify_no_tcp.sh            # TCP verification (CRITICAL!)
├── GETTING_STARTED.md          # This file
├── components/net/
│   ├── transport_url.rs        # URL parser
│   ├── unix_connector.rs       # Socket connector
│   ├── http_loader.rs          # Request routing
│   └── ...
├── examples/
│   ├── unix_socket_server.py   # Flask test server
│   └── run_gunicorn.sh         # Gunicorn launcher
├── tests/
│   └── uds_integration_test.rs # Rust integration tests
├── test_uds_python.py          # Python integration tests
└── test_uds_integration.sh     # Bash integration tests
```

## Performance Tips

### Faster Builds

```bash
# Use ccache
export CCACHE_DIR=~/.ccache
export RUSTC_WRAPPER=sccache

# Parallel builds
./mach build -j 8  # Use 8 cores
```

### Development Workflow

```bash
# Watch for changes and rebuild
cargo watch -x "check -p net"

# Run specific tests only
cargo test --lib -p net transport_url::tests::test_parse_unix_socket_url

# Format code automatically
cargo fmt --all

# Fix clippy warnings
cargo clippy --all --fix
```

## Security Considerations

### Why No TCP?

Unix Domain Sockets provide several security advantages over TCP:

1. **Filesystem Permissions** - Access controlled by file permissions
2. **No Network Exposure** - Cannot be accessed remotely
3. **Process Credentials** - Can verify connecting process UID/GID
4. **Lower Attack Surface** - No network stack vulnerabilities

### Socket Permissions

```bash
# Restrict socket access to owner only
chmod 600 /tmp/private.sock

# Allow group access
chmod 660 /tmp/shared.sock
chgrp mygroup /tmp/shared.sock

# World-readable (for development only!)
chmod 666 /tmp/public.sock
```

### Production Deployment

For production use:

```bash
# Use dedicated socket directory
mkdir -p /var/run/myapp
chown myapp:myapp /var/run/myapp
chmod 750 /var/run/myapp

# Start Gunicorn as dedicated user
sudo -u myapp gunicorn --bind unix:/var/run/myapp/app.sock myapp:app
```

## Next Steps

1. **Read Implementation Docs**
   - `TRANSPORT_URL_IMPLEMENTATION.md` - URL parser details
   - `UDS_INTEGRATION_PROGRESS.md` - Project status

2. **Explore Examples**
   - `examples/unix_socket_server.py` - Flask server code
   - `test_uds_python.py` - Test examples

3. **Customize Configuration**
   - Modify `examples/unix_socket_server.py` for your application
   - Add custom routes and endpoints

4. **Build Your Application**
   - Use Unix sockets for microservices
   - Integrate with containers
   - Create local development setups

## Support

If you encounter issues:

1. Check test logs: `/tmp/servo-uds-tests-*.log`
2. Check verification report: `/tmp/servo-tcp-verification-*.txt`
3. Enable debug logging: `export RUST_LOG=net=debug`
4. Review troubleshooting section above

## License

This implementation follows Servo's Mozilla Public License 2.0.
