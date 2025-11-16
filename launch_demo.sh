#!/bin/bash
# Automated Demo Launcher for Servo UDS
# Starts Gunicorn server and Servo browser, automatically navigates to test page

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SOCKET_PATH="/tmp/servo-demo.sock"
SOCKET_DIR="/tmp/servo-sockets"
PID_FILE="/tmp/servo-demo-gunicorn.pid"
SCREENSHOT_PATH="/tmp/servo-demo-screenshot.png"
DEMO_URL="http::unix///${SOCKET_PATH}"

echo "=========================================="
echo "Servo UDS Demo Launcher"
echo "=========================================="
echo

# Function to cleanup on exit
cleanup() {
    echo
    echo -e "${BLUE}Cleaning up...${NC}"

    # Kill Gunicorn
    if [ -f "$PID_FILE" ]; then
        echo "Stopping Gunicorn..."
        kill "$(cat $PID_FILE)" 2>/dev/null || true
        rm -f "$PID_FILE"
    fi

    # Remove sockets
    rm -f "$SOCKET_PATH"

    echo -e "${GREEN}✓ Cleanup complete${NC}"
}

trap cleanup EXIT

# Function to check dependencies
check_dependencies() {
    echo -e "${BLUE}Checking dependencies...${NC}"

    local all_good=true

    # Check Python
    if ! command -v python3 >/dev/null 2>&1; then
        echo -e "${RED}✗ python3 not found${NC}"
        all_good=false
    fi

    # Check Gunicorn
    if ! python3 -c "import gunicorn" 2>/dev/null; then
        echo -e "${RED}✗ Gunicorn not installed${NC}"
        echo "  Install with: pip3 install gunicorn flask"
        all_good=false
    fi

    # Check Flask app
    if [ ! -f "examples/unix_socket_server.py" ]; then
        echo -e "${RED}✗ Flask app not found: examples/unix_socket_server.py${NC}"
        all_good=false
    fi

    # Check Servo (optional - can run in headless mode without full build)
    if [ -f "./target/debug/servo" ] || [ -f "./target/release/servo" ]; then
        echo -e "${GREEN}✓ Servo binary found${NC}"
    else
        echo -e "${YELLOW}⚠ Servo binary not found - will skip browser launch${NC}"
        echo "  Build with: ./mach build -d"
    fi

    # Check curl for testing
    if ! command -v curl >/dev/null 2>&1; then
        echo -e "${RED}✗ curl not found${NC}"
        all_good=false
    fi

    # Check network tools
    if ! command -v ss >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠ ss not found - will skip TCP verification${NC}"
    fi

    if [ "$all_good" = false ]; then
        echo
        echo -e "${RED}Missing required dependencies!${NC}"
        echo "Run: ./setup_dev_environment.sh"
        exit 1
    fi

    echo -e "${GREEN}✓ All required dependencies found${NC}"
    echo
}

# Function to start Gunicorn
start_gunicorn() {
    echo -e "${BLUE}Starting Gunicorn server...${NC}"

    # Create socket directory
    mkdir -p "$SOCKET_DIR"

    # Remove old socket if exists
    rm -f "$SOCKET_PATH"

    # Start Gunicorn
    cd examples
    gunicorn \
        --bind "unix:$SOCKET_PATH" \
        --workers 2 \
        --access-logfile - \
        --error-logfile - \
        --log-level info \
        --daemon \
        --pid "$PID_FILE" \
        unix_socket_server:app

    cd ..

    # Wait for socket to be created
    local max_wait=10
    local waited=0
    while [ ! -S "$SOCKET_PATH" ] && [ $waited -lt $max_wait ]; do
        sleep 1
        ((waited++))
    done

    if [ ! -S "$SOCKET_PATH" ]; then
        echo -e "${RED}✗ Failed to start Gunicorn - socket not created${NC}"
        exit 1
    fi

    echo -e "${GREEN}✓ Gunicorn started on Unix socket: $SOCKET_PATH${NC}"
    echo
}

# Function to verify no TCP connections
verify_no_tcp() {
    echo -e "${BLUE}Verifying no TCP connections...${NC}"

    local has_tcp=false

    # Check for TCP listeners
    if ss -tlnp 2>/dev/null | grep -q gunicorn; then
        echo -e "${RED}✗ WARNING: Found TCP listeners!${NC}"
        ss -tlnp | grep gunicorn
        has_tcp=true
    else
        echo -e "${GREEN}✓ No TCP listeners${NC}"
    fi

    # Check for TCP connections
    if ss -tnp 2>/dev/null | grep -q gunicorn; then
        echo -e "${RED}✗ WARNING: Found TCP connections!${NC}"
        ss -tnp | grep gunicorn
        has_tcp=true
    else
        echo -e "${GREEN}✓ No TCP connections${NC}"
    fi

    # Check Unix socket
    if lsof "$SOCKET_PATH" 2>/dev/null | grep -q gunicorn; then
        echo -e "${GREEN}✓ Unix socket is open by Gunicorn${NC}"
    else
        echo -e "${YELLOW}⚠ Could not verify Unix socket (lsof may require sudo)${NC}"
    fi

    if [ "$has_tcp" = true ]; then
        echo
        echo -e "${RED}=========================================="
        echo "⚠ WARNING: TCP connections detected!"
        echo "This violates the UDS-only requirement."
        echo "==========================================${NC}"
        echo
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi

    echo
}

# Function to test with curl
test_with_curl() {
    echo -e "${BLUE}Testing server with curl...${NC}"

    # Test root page
    echo "GET /"
    if curl -s --unix-socket "$SOCKET_PATH" http://localhost/ | grep -q "Hello from Unix Socket Server"; then
        echo -e "${GREEN}✓ Root page working${NC}"
    else
        echo -e "${RED}✗ Root page failed${NC}"
        return 1
    fi

    # Test API endpoint
    echo "GET /api/data"
    if curl -s --unix-socket "$SOCKET_PATH" http://localhost/api/data | grep -q "unix_domain_socket"; then
        echo -e "${GREEN}✓ API endpoint working${NC}"
    else
        echo -e "${RED}✗ API endpoint failed${NC}"
        return 1
    fi

    echo
}

# Function to launch Servo
launch_servo() {
    echo -e "${BLUE}Launching Servo browser...${NC}"
    echo

    # Find Servo binary
    local servo_bin=""
    if [ -f "./target/debug/servo" ]; then
        servo_bin="./target/debug/servo"
    elif [ -f "./target/release/servo" ]; then
        servo_bin="./target/release/servo"
    else
        echo -e "${YELLOW}⚠ Servo binary not found - skipping browser launch${NC}"
        echo "  Build Servo with: ./mach build -d"
        return 0
    fi

    echo "Servo binary: $servo_bin"
    echo "Demo URL: $DEMO_URL"
    echo

    # Set environment variables for Unix socket mode
    export SERVO_USE_UNIX_SOCKETS=1
    export SERVO_SOCKET_DIR="$SOCKET_DIR"
    export SERVO_SOCKET_MAPPINGS="localhost:$SOCKET_PATH"
    export RUST_LOG=net=debug

    echo "Environment:"
    echo "  SERVO_USE_UNIX_SOCKETS=$SERVO_USE_UNIX_SOCKETS"
    echo "  SERVO_SOCKET_DIR=$SERVO_SOCKET_DIR"
    echo "  SERVO_SOCKET_MAPPINGS=$SERVO_SOCKET_MAPPINGS"
    echo

    # Ask user for mode
    echo "Choose demo mode:"
    echo "  1) Headless (screenshot)"
    echo "  2) GUI (interactive)"
    echo "  3) Skip Servo launch"
    read -p "Select mode (1/2/3): " -n 1 -r
    echo

    case $REPLY in
        1)
            echo -e "${BLUE}Running in headless mode...${NC}"
            echo "Screenshot will be saved to: $SCREENSHOT_PATH"
            echo

            # Run headless with screenshot
            "$servo_bin" -z -o "$SCREENSHOT_PATH" "$DEMO_URL"

            # Check if screenshot was created
            if [ -f "$SCREENSHOT_PATH" ]; then
                local size=$(stat -f%z "$SCREENSHOT_PATH" 2>/dev/null || stat -c%s "$SCREENSHOT_PATH")
                echo
                echo -e "${GREEN}✓ Screenshot created: $SCREENSHOT_PATH ($size bytes)${NC}"
                echo
                echo "View with: xdg-open $SCREENSHOT_PATH"
            else
                echo -e "${RED}✗ Screenshot not created${NC}"
            fi
            ;;
        2)
            echo -e "${BLUE}Launching Servo in GUI mode...${NC}"
            echo "Press Ctrl+C to stop"
            echo

            # Run interactive
            "$servo_bin" "$DEMO_URL"
            ;;
        *)
            echo "Skipping Servo launch"
            ;;
    esac

    echo
}

# Function to show server logs
show_logs() {
    echo -e "${BLUE}=========================================="
    echo "Server Information"
    echo "==========================================${NC}"
    echo
    echo "Socket path: $SOCKET_PATH"
    echo "PID file: $PID_FILE"
    echo "Gunicorn PID: $(cat $PID_FILE 2>/dev/null || echo 'N/A')"
    echo
    echo "Test the server manually with:"
    echo "  curl --unix-socket $SOCKET_PATH http://localhost/"
    echo "  curl --unix-socket $SOCKET_PATH http://localhost/api/data"
    echo
    echo "Access via Servo with:"
    echo "  export SERVO_USE_UNIX_SOCKETS=1"
    echo "  export SERVO_SOCKET_MAPPINGS=\"localhost:$SOCKET_PATH\""
    echo "  servo \"$DEMO_URL\""
    echo
}

# Main demo flow
main() {
    check_dependencies
    start_gunicorn
    verify_no_tcp
    test_with_curl

    # Keep server running
    if [ "$1" = "--server-only" ]; then
        echo -e "${GREEN}=========================================="
        echo "Server running in background"
        echo "==========================================${NC}"
        show_logs
        echo "Press Ctrl+C to stop the server"
        echo

        # Wait for interrupt
        trap - EXIT  # Don't cleanup on normal exit
        while true; do
            sleep 1
        done
    else
        launch_servo
        show_logs

        echo -e "${GREEN}=========================================="
        echo "Demo complete!"
        echo "==========================================${NC}"
    fi
}

# Parse command line arguments
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  --server-only    Start server only, don't launch Servo"
    echo "  --help, -h       Show this help message"
    echo
    echo "This script will:"
    echo "  1. Start Gunicorn on a Unix domain socket"
    echo "  2. Verify no TCP connections are made"
    echo "  3. Test the server with curl"
    echo "  4. Launch Servo browser (optional)"
    echo
    exit 0
fi

# Run main
main "$@"
