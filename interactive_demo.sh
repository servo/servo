#!/bin/bash
#
# Interactive Servo UDS Demo
#
# This script provides a streamlined interactive demo of Servo's Unix Domain Socket networking:
# 1. Starts Gunicorn server on a Unix socket (background)
# 2. Launches Servo browser in GUI mode for interactive testing
# 3. Automatically cleans up on exit
#
# Usage:
#   ./interactive_demo.sh              # Start demo (GUI mode)
#   ./interactive_demo.sh --headless   # Run in headless mode (screenshot)
#   ./interactive_demo.sh --help       # Show help

set -e

# Configuration
SOCKET_DIR="${SERVO_SOCKET_DIR:-/tmp/servo-sockets}"
SOCKET_PATH="$SOCKET_DIR/localhost.sock"
URL="${URL:-http::unix//localhost/}"
MODE="${1:-gui}"

# Colors for pretty output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Banner
print_banner() {
    echo -e "${CYAN}╔════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                                                ║${NC}"
    echo -e "${CYAN}║${NC}     ${BOLD}Servo Unix Domain Socket Demo${NC}          ${CYAN}║${NC}"
    echo -e "${CYAN}║                                                ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════╝${NC}"
    echo
}

# Help message
show_help() {
    print_banner
    echo "A streamlined interactive demo of Servo's Unix socket networking."
    echo
    echo -e "${BOLD}Usage:${NC}"
    echo "  $0 [OPTIONS]"
    echo
    echo -e "${BOLD}Options:${NC}"
    echo "  --headless    Run Servo in headless mode (screenshot)"
    echo "  --help        Show this help message"
    echo
    echo -e "${BOLD}What this demo does:${NC}"
    echo "  1. Starts a Gunicorn web server on a Unix domain socket"
    echo "  2. Launches Servo browser to connect via the socket"
    echo "  3. Demonstrates pure UDS networking (no TCP/IP!)"
    echo "  4. Automatically cleans up when you exit"
    echo
    echo -e "${BOLD}Environment Variables:${NC}"
    echo "  SERVO_SOCKET_DIR   Socket directory (default: /tmp/servo-sockets)"
    echo "  URL                URL to load (default: http::unix//localhost/)"
    echo
    echo -e "${BOLD}Examples:${NC}"
    echo "  ./interactive_demo.sh                                  # Interactive GUI demo"
    echo "  ./interactive_demo.sh --headless                       # Headless screenshot"
    echo "  URL=http::unix//localhost/test ./interactive_demo.sh   # Load /test page"
    echo
    exit 0
}

# Parse arguments
if [ "$MODE" = "--help" ] || [ "$MODE" = "-h" ]; then
    show_help
fi

# Cleanup function
cleanup() {
    echo
    echo -e "${YELLOW}🧹 Cleaning up...${NC}"

    # Stop Gunicorn
    if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
        echo "  Stopping Gunicorn (PID: $SERVER_PID)..."
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi

    # Remove socket
    if [ -S "$SOCKET_PATH" ]; then
        echo "  Removing socket: $SOCKET_PATH"
        rm -f "$SOCKET_PATH"
    fi

    echo -e "${GREEN}✓ Cleanup complete${NC}"
    echo
    echo -e "${CYAN}Thanks for trying the Servo UDS demo! 🚀${NC}"
}

trap cleanup EXIT INT TERM

# Check dependencies
check_dependencies() {
    local missing=0

    echo -e "${BLUE}📋 Checking dependencies...${NC}"

    # Check Python
    if ! command -v python3 >/dev/null 2>&1; then
        echo -e "${RED}  ✗ python3 not found${NC}"
        missing=1
    else
        echo -e "${GREEN}  ✓ python3${NC}"
    fi

    # Check Gunicorn
    if ! python3 -c "import gunicorn" 2>/dev/null; then
        echo -e "${RED}  ✗ gunicorn not installed${NC}"
        echo -e "    Install with: ${CYAN}pip3 install -r examples/requirements.txt${NC}"
        missing=1
    else
        echo -e "${GREEN}  ✓ gunicorn${NC}"
    fi

    # Check Flask
    if ! python3 -c "import flask" 2>/dev/null; then
        echo -e "${RED}  ✗ flask not installed${NC}"
        echo -e "    Install with: ${CYAN}pip3 install -r examples/requirements.txt${NC}"
        missing=1
    else
        echo -e "${GREEN}  ✓ flask${NC}"
    fi

    # Check Flask app
    if [ ! -f "examples/unix_socket_server.py" ]; then
        echo -e "${RED}  ✗ Flask app not found: examples/unix_socket_server.py${NC}"
        missing=1
    else
        echo -e "${GREEN}  ✓ Flask app${NC}"
    fi

    if [ $missing -eq 1 ]; then
        echo
        echo -e "${RED}❌ Missing required dependencies!${NC}"
        echo -e "   Run: ${CYAN}pip3 install -r examples/requirements.txt${NC}"
        exit 1
    fi

    echo
}

# Start Gunicorn server
start_server() {
    echo -e "${BLUE}🚀 Starting Gunicorn server...${NC}"

    # Create socket directory
    mkdir -p "$SOCKET_DIR"

    # Remove old socket if exists
    if [ -S "$SOCKET_PATH" ]; then
        echo -e "${YELLOW}  Removing old socket...${NC}"
        rm -f "$SOCKET_PATH"
    fi

    # Start Gunicorn in background
    cd examples
    gunicorn \
        --bind "unix:$SOCKET_PATH" \
        --workers 2 \
        --worker-class sync \
        --access-logfile - \
        --error-logfile - \
        --log-level info \
        --timeout 120 \
        unix_socket_server:app &

    SERVER_PID=$!
    cd ..

    echo -e "${GREEN}  ✓ Gunicorn started (PID: $SERVER_PID)${NC}"

    # Wait for socket to be ready
    echo -e "${YELLOW}  Waiting for socket...${NC}"
    local attempts=0
    local max_attempts=30

    while [ ! -S "$SOCKET_PATH" ] && [ $attempts -lt $max_attempts ]; do
        sleep 0.5
        attempts=$((attempts + 1))

        # Check if server process is still running
        if ! kill -0 "$SERVER_PID" 2>/dev/null; then
            echo -e "${RED}  ✗ Gunicorn process died unexpectedly${NC}"
            exit 1
        fi
    done

    if [ ! -S "$SOCKET_PATH" ]; then
        echo -e "${RED}  ✗ Timeout waiting for socket${NC}"
        exit 1
    fi

    echo -e "${GREEN}  ✓ Socket ready: $SOCKET_PATH${NC}"
    echo
}

# Test server with curl
test_server() {
    echo -e "${BLUE}🧪 Testing server...${NC}"

    if ! command -v curl >/dev/null 2>&1; then
        echo -e "${YELLOW}  ⚠ curl not found, skipping test${NC}"
        echo
        return
    fi

    # Test root endpoint
    if curl -s --unix-socket "$SOCKET_PATH" http://localhost/ | grep -q "Unix Socket Server" 2>/dev/null; then
        echo -e "${GREEN}  ✓ Server responding correctly${NC}"
    else
        echo -e "${YELLOW}  ⚠ Server test inconclusive${NC}"
    fi

    echo
}

# Find Servo binary
find_servo() {
    if [ -f "target/release/servo" ]; then
        echo "target/release/servo"
    elif [ -f "target/debug/servo" ]; then
        echo "target/debug/servo"
    else
        echo ""
    fi
}

# Launch Servo browser
launch_servo() {
    echo -e "${BLUE}🌐 Launching Servo browser...${NC}"

    # Find Servo binary
    SERVO_BIN=$(find_servo)

    if [ -z "$SERVO_BIN" ]; then
        echo -e "${RED}  ✗ Servo binary not found${NC}"
        echo
        echo -e "${YELLOW}  Build Servo first:${NC}"
        echo -e "    ${CYAN}./mach build${NC}         # Release build (recommended)"
        echo -e "    ${CYAN}./mach build -d${NC}      # Debug build (faster compilation)"
        echo
        exit 1
    fi

    echo -e "${GREEN}  ✓ Found Servo: $SERVO_BIN${NC}"
    echo

    # UDS mode is now enabled by default in Servo!
    # No environment variables needed
    export RUST_LOG="${RUST_LOG:-net=debug}"

    # Show configuration
    echo -e "${CYAN}════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}Demo Configuration:${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════${NC}"
    echo -e "  UDS Mode:          ${GREEN}Enabled (default)${NC}"
    echo -e "  Socket directory:  ${YELLOW}$SOCKET_DIR${NC}"
    echo -e "  Socket path:       ${YELLOW}$SOCKET_PATH${NC}"
    echo -e "  URL to load:       ${YELLOW}$URL${NC}"
    echo -e "  Servo binary:      ${YELLOW}$SERVO_BIN${NC}"

    if [ "$MODE" = "--headless" ]; then
        echo -e "  Mode:              ${YELLOW}Headless (screenshot)${NC}"
        echo -e "${CYAN}════════════════════════════════════════════════${NC}"
        echo
        echo -e "${GREEN}📸 Taking screenshot...${NC}"

        SCREENSHOT="/tmp/servo-uds-demo-$(date +%s).png"
        "$SERVO_BIN" -z -x -o "$SCREENSHOT" "$URL"

        if [ -f "$SCREENSHOT" ]; then
            echo
            echo -e "${GREEN}✓ Screenshot saved: $SCREENSHOT${NC}"
            echo -e "  View with: ${CYAN}xdg-open $SCREENSHOT${NC}"
        else
            echo -e "${RED}✗ Screenshot failed${NC}"
        fi
    else
        echo -e "  Mode:              ${YELLOW}Interactive GUI${NC}"
        echo -e "${CYAN}════════════════════════════════════════════════${NC}"
        echo
        echo -e "${GREEN}🎨 Launching interactive browser...${NC}"
        echo -e "${YELLOW}   (Press Ctrl+C to exit and cleanup)${NC}"
        echo
        echo -e "${CYAN}════════════════════════════════════════════════${NC}"
        echo -e "${BOLD}Server Logs:${NC}"
        echo -e "${CYAN}════════════════════════════════════════════════${NC}"

        # Launch Servo in GUI mode
        "$SERVO_BIN" "$URL"
    fi

    echo
}

# Main demo flow
main() {
    print_banner

    echo -e "${CYAN}This demo shows Servo using ${BOLD}only${NC}${CYAN} Unix Domain Sockets.${NC}"
    echo -e "${CYAN}No TCP/IP networking is used! 🔒${NC}"
    echo

    check_dependencies
    start_server
    test_server
    launch_servo

    echo -e "${GREEN}════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}Demo completed successfully! ✨${NC}"
    echo -e "${GREEN}════════════════════════════════════════════════${NC}"
    echo
}

# Run the demo
main
