#!/bin/bash
#
# Demo: Run Servo with Unix Domain Socket Server
#
# This script demonstrates Servo's Unix socket networking by:
# 1. Starting a Gunicorn server on a Unix socket
# 2. Launching Servo (headless or GUI) to connect via UDS
#

set -e

# Configuration
SOCKET_DIR="${SERVO_SOCKET_DIR:-/tmp/servo-sockets}"
SOCKET_PATH="$SOCKET_DIR/localhost.sock"
HEADLESS="${HEADLESS:-false}"
URL="${1:-http://localhost/}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Servo Unix Domain Socket Demo${NC}"
echo -e "${BLUE}========================================${NC}"
echo

# Cleanup function
cleanup() {
    echo
    echo -e "${YELLOW}Stopping server...${NC}"
    if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
    rm -f "$SOCKET_PATH"
    echo -e "${GREEN}Done${NC}"
}

trap cleanup EXIT INT TERM

# Check if server is already running
if [ -S "$SOCKET_PATH" ]; then
    echo -e "${YELLOW}Socket already exists: $SOCKET_PATH${NC}"
    echo -e "${YELLOW}Removing old socket...${NC}"
    rm -f "$SOCKET_PATH"
fi

# Start server
echo -e "${GREEN}Starting Gunicorn server...${NC}"
mkdir -p "$SOCKET_DIR"

cd examples
gunicorn \
    --bind "unix:$SOCKET_PATH" \
    --workers 2 \
    --access-logfile - \
    --error-logfile - \
    --log-level info \
    unix_socket_server:app &

SERVER_PID=$!
cd ..

# Wait for socket
echo -e "${YELLOW}Waiting for socket to be ready...${NC}"
for i in {1..20}; do
    if [ -S "$SOCKET_PATH" ]; then
        echo -e "${GREEN}✓ Server ready on socket: $SOCKET_PATH${NC}"
        break
    fi
    sleep 0.5
    if [ $i -eq 20 ]; then
        echo -e "${RED}Timeout waiting for socket${NC}"
        exit 1
    fi
done

echo

# Find Servo binary
SERVO_BIN=""
if [ -f "target/release/servo" ]; then
    SERVO_BIN="target/release/servo"
elif [ -f "target/debug/servo" ]; then
    SERVO_BIN="target/debug/servo"
else
    echo -e "${RED}Servo binary not found. Please build Servo first:${NC}"
    echo -e "  ${BLUE}./mach build${NC}"
    exit 1
fi

echo -e "${GREEN}Using Servo: $SERVO_BIN${NC}"
echo

# Configure environment
export SERVO_USE_UNIX_SOCKETS=true
export SERVO_SOCKET_DIR="$SOCKET_DIR"
export SERVO_SOCKET_MAPPINGS="localhost:$SOCKET_PATH"
export RUST_LOG="${RUST_LOG:-net=debug}"

echo -e "${BLUE}Configuration:${NC}"
echo -e "  Socket directory: ${YELLOW}$SOCKET_DIR${NC}"
echo -e "  Socket path:      ${YELLOW}$SOCKET_PATH${NC}"
echo -e "  URL:              ${YELLOW}$URL${NC}"
echo -e "  Mode:             ${YELLOW}$([ "$HEADLESS" = "true" ] && echo "Headless" || echo "GUI")${NC}"
echo
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}✓ Server is ready!${NC}"
echo
echo -e "${YELLOW}To test the server manually, type this URL in Servo:${NC}"
echo -e "${GREEN}  $URL${NC}"
echo
echo -e "${YELLOW}Or use curl to test:${NC}"
echo -e "  ${BLUE}curl --unix-socket $SOCKET_PATH http://localhost/${NC}"
echo -e "${BLUE}========================================${NC}"
echo

echo -e "${GREEN}Launching Servo...${NC}"
echo -e "${YELLOW}(Server logs will appear below)${NC}"
echo
echo "========================================"

# Run Servo
if [ "$HEADLESS" = "true" ]; then
    "$SERVO_BIN" -z -x -o /tmp/servo-uds-demo.png "$URL"
    echo
    echo -e "${GREEN}Screenshot saved to: /tmp/servo-uds-demo.png${NC}"
else
    "$SERVO_BIN" "$URL"
fi
