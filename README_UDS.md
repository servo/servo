# Servo Unix Domain Socket (UDS) Networking

This modified version of Servo supports accessing websites over Unix domain sockets (IPC) instead of TCP connections.

## Quick Start

### 1. Build Servo

```bash
./mach build
```

### 2. Run the Test

```bash
./test_uds.sh
```

### 3. Run the Demo

Terminal 1 - Start the server:
```bash
cd examples
./run_gunicorn.sh
```

Terminal 2 - Run Servo (headless):
```bash
export SERVO_USE_UNIX_SOCKETS=true
export SERVO_SOCKET_DIR=/tmp/servo-sockets
export SERVO_SOCKET_MAPPINGS="localhost:/tmp/servo-sockets/localhost.sock"
./servo -z -x -o output.png http://localhost/
```

Or use the demo script:
```bash
HEADLESS=true ./demo_uds.sh
```

## Command Line Options

### Using CLI Flags

```bash
./servo --unix-sockets \
        --socket-dir=/tmp/servo-sockets \
        --socket-mapping="localhost:/tmp/servo-sockets/localhost.sock" \
        -z http://localhost/
```

### Using Environment Variables

```bash
export SERVO_USE_UNIX_SOCKETS=true
export SERVO_SOCKET_DIR=/tmp/servo-sockets
export SERVO_SOCKET_MAPPINGS="localhost:/path/to/socket,example.com:/another/socket"
./servo -z http://localhost/
```

## Options

| CLI Flag | Environment Variable | Description |
|----------|---------------------|-------------|
| `--unix-sockets` | `SERVO_USE_UNIX_SOCKETS=true` | Enable Unix socket mode |
| `--socket-dir=<path>` | `SERVO_SOCKET_DIR=<path>` | Default socket directory |
| `--socket-mapping=<map>` | `SERVO_SOCKET_MAPPINGS=<map>` | Hostname-to-socket mappings |

## Example Server

A Flask + Gunicorn example server is provided in `examples/`:

```bash
# Install dependencies
pip install -r examples/requirements.txt

# Run server
cd examples && ./run_gunicorn.sh
```

## Architecture

- **URL Mapping**: `http://localhost/` → `/tmp/servo-sockets/localhost.sock`
- **Custom Mappings**: Map any hostname to a socket path
- **No TCP**: All connections go through Unix sockets (no loopback, no network stack)
- **Logging**: Set `RUST_LOG=net=trace` to see connection details

## Files Modified

- `components/net/Cargo.toml` - Added hyperlocal dependency
- `components/net/unix_config.rs` - UDS configuration
- `components/net/unix_connector.rs` - Unix socket connector
- `components/net/connector.rs` - Multi-mode connector support
- `components/net/resource_thread.rs` - UDS integration
- `ports/servoshell/prefs.rs` - CLI flags

## Testing

### Automated Test
```bash
./test_uds.sh
```

### Manual Test with Logging
```bash
# Terminal 1
cd examples && ./run_gunicorn.sh

# Terminal 2
RUST_LOG=net=trace \
SERVO_USE_UNIX_SOCKETS=true \
./servo -z -o test.png http://localhost/
```

### Verify Network Isolation
```bash
# This should work (UDS)
SERVO_USE_UNIX_SOCKETS=true ./servo -z http://localhost/

# This would fail in UDS-only mode (no TCP fallback)
SERVO_USE_UNIX_SOCKETS=true ./servo -z http://example.com/
```

## Troubleshooting

**Socket not found**
- Check `SERVO_SOCKET_DIR` matches where Gunicorn creates the socket
- Verify socket file exists: `ls -la /tmp/servo-sockets/`

**Permission denied**
- Socket file permissions: `chmod 666 /tmp/servo-sockets/localhost.sock`
- Or run Gunicorn as same user as Servo

**Connection refused**
- Ensure Gunicorn is running: `ps aux | grep gunicorn`
- Check socket exists: `test -S /tmp/servo-sockets/localhost.sock && echo exists`

**Logging**
```bash
# Network layer debug
RUST_LOG=net=debug ./servo -z http://localhost/

# Full trace
RUST_LOG=net=trace ./servo -z http://localhost/
```
