#!/bin/bash
#
# Start UDS Server
#
# Starts the Gunicorn server on a Unix Domain Socket as a long-running process.
# This script starts the server and keeps it running until you stop it.
#
# Usage:
#   ./start_uds_server.sh           # Start server
#   ./start_uds_server.sh --help    # Show help
#

set -e

# Configuration
SOCKET_DIR="${SERVO_SOCKET_DIR:-/tmp/servo-sockets}"
SOCKET_PATH="$SOCKET_DIR/localhost.sock"
PID_FILE="/tmp/servo-uds-server.pid"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Help message
show_help() {
    echo -e "${CYAN}╔════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC}  ${BOLD}Start Unix Domain Socket Server${NC}            ${CYAN}║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════╝${NC}"
    echo
    echo "Starts Gunicorn server on a Unix socket as a long-running process."
    echo
    echo -e "${BOLD}Usage:${NC}"
    echo "  $0 [OPTIONS]"
    echo
    echo -e "${BOLD}Options:${NC}"
    echo "  --help        Show this help message"
    echo
    echo -e "${BOLD}Environment Variables:${NC}"
    echo "  SERVO_SOCKET_DIR   Socket directory (default: /tmp/servo-sockets)"
    echo
    echo -e "${BOLD}What this does:${NC}"
    echo "  1. Checks dependencies (Python, Gunicorn, Flask)"
    echo "  2. Starts Gunicorn server on Unix socket"
    echo "  3. Prints the URL to use for testing"
    echo "  4. Keeps the server running until Ctrl+C"
    echo
    echo -e "${BOLD}Testing:${NC}"
    echo "  After starting, you can test with:"
    echo "    - Servo browser (run ./launch_servo.sh)"
    echo "    - curl --unix-socket $SOCKET_PATH http://localhost/"
    echo
    exit 0
}

# Parse arguments
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    show_help
fi

# Cleanup function
cleanup() {
    echo
    echo -e "${YELLOW}🛑 Stopping server...${NC}"

    # Stop Gunicorn
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE" 2>/dev/null)
        if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            echo "  Stopping Gunicorn (PID: $pid)..."
            kill "$pid" 2>/dev/null || true
            # Wait for process to die
            local count=0
            while kill -0 "$pid" 2>/dev/null && [ $count -lt 10 ]; do
                sleep 0.5
                count=$((count + 1))
            done
            # Force kill if still running
            if kill -0 "$pid" 2>/dev/null; then
                kill -9 "$pid" 2>/dev/null || true
            fi
        fi
        rm -f "$PID_FILE"
    fi

    # Remove socket
    if [ -S "$SOCKET_PATH" ]; then
        echo "  Removing socket: $SOCKET_PATH"
        rm -f "$SOCKET_PATH"
    fi

    echo -e "${GREEN}✓ Server stopped${NC}"
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

# Start server
start_server() {
    echo -e "${BLUE}🚀 Starting Gunicorn server...${NC}"

    # Create socket directory
    mkdir -p "$SOCKET_DIR"

    # Check if server already running
    if [ -f "$PID_FILE" ]; then
        local old_pid=$(cat "$PID_FILE" 2>/dev/null)
        if [ -n "$old_pid" ] && kill -0 "$old_pid" 2>/dev/null; then
            echo -e "${YELLOW}  ⚠ Server already running (PID: $old_pid)${NC}"
            echo -e "${YELLOW}  Stopping old server first...${NC}"
            kill "$old_pid" 2>/dev/null || true
            sleep 1
        fi
        rm -f "$PID_FILE"
    fi

    # Remove old socket if exists
    if [ -S "$SOCKET_PATH" ]; then
        echo -e "${YELLOW}  Removing old socket...${NC}"
        rm -f "$SOCKET_PATH"
    fi

    # Start Gunicorn
    cd examples
    gunicorn \
        --bind "unix:$SOCKET_PATH" \
        --workers 2 \
        --worker-class sync \
        --access-logfile - \
        --error-logfile - \
        --log-level info \
        --timeout 120 \
        --pid "$PID_FILE" \
        unix_socket_server:app &

    local server_pid=$!
    cd ..

    # Save PID
    echo "$server_pid" > "$PID_FILE"

    echo -e "${GREEN}  ✓ Gunicorn started (PID: $server_pid)${NC}"

    # Wait for socket to be ready
    echo -e "${YELLOW}  Waiting for socket...${NC}"
    local attempts=0
    local max_attempts=30

    while [ ! -S "$SOCKET_PATH" ] && [ $attempts -lt $max_attempts ]; do
        sleep 0.5
        attempts=$((attempts + 1))

        # Check if server process is still running
        if ! kill -0 "$server_pid" 2>/dev/null; then
            echo -e "${RED}  ✗ Gunicorn process died unexpectedly${NC}"
            rm -f "$PID_FILE"
            exit 1
        fi
    done

    if [ ! -S "$SOCKET_PATH" ]; then
        echo -e "${RED}  ✗ Timeout waiting for socket${NC}"
        rm -f "$PID_FILE"
        exit 1
    fi

    echo -e "${GREEN}  ✓ Socket ready: $SOCKET_PATH${NC}"
    echo
}

# Test server
test_server() {
    if ! command -v curl >/dev/null 2>&1; then
        return
    fi

    echo -e "${BLUE}🧪 Testing server...${NC}"

    if curl -s --unix-socket "$SOCKET_PATH" http://localhost/ | grep -q "Unix Socket Server" 2>/dev/null; then
        echo -e "${GREEN}  ✓ Server responding correctly${NC}"
    else
        echo -e "${YELLOW}  ⚠ Server test inconclusive${NC}"
    fi

    echo
}

# Show connection info
show_info() {
    echo -e "${CYAN}╔════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC}  ${BOLD}Server Running Successfully!${NC}                ${CYAN}║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════╝${NC}"
    echo
    echo -e "${BOLD}Server Details:${NC}"
    echo -e "  Socket:       ${GREEN}$SOCKET_PATH${NC}"
    echo -e "  PID:          ${GREEN}$(cat $PID_FILE)${NC}"
    echo -e "  PID File:     ${GREEN}$PID_FILE${NC}"
    echo
    echo -e "${CYAN}════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}How to Test:${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════${NC}"
    echo
    echo -e "${YELLOW}1. Using Servo browser:${NC}"
    echo -e "   Run this command in another terminal:"
    echo -e "   ${GREEN}./launch_servo.sh${NC}"
    echo
    echo -e "   ${BOLD}Or type this URL in Servo:${NC}"
    echo -e "   ${GREEN}http://localhost/${NC}"
    echo
    echo -e "${YELLOW}2. Using curl:${NC}"
    echo -e "   ${BLUE}curl --unix-socket $SOCKET_PATH http://localhost/${NC}"
    echo -e "   ${BLUE}curl --unix-socket $SOCKET_PATH http://localhost/api/data${NC}"
    echo -e "   ${BLUE}curl --unix-socket $SOCKET_PATH http://localhost/test${NC}"
    echo
    echo -e "${CYAN}════════════════════════════════════════════════${NC}"
    echo
    echo -e "${YELLOW}Server is running. Press Ctrl+C to stop.${NC}"
    echo
    echo -e "${BLUE}Server logs:${NC}"
    echo "------------------------------------------------"
}

# Main function
main() {
    echo -e "${CYAN}╔════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC}  ${BOLD}Unix Domain Socket Server Launcher${NC}         ${CYAN}║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════╝${NC}"
    echo

    check_dependencies
    start_server
    test_server
    show_info

    # Keep running and showing logs
    # The server is running in background, we just wait here
    while true; do
        sleep 1
    done
}

# Run
main
