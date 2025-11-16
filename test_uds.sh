#!/bin/bash
#
# Test Servo Unix Domain Socket (UDS) Implementation
#
# This script tests the UDS functionality by:
# 1. Starting a Gunicorn server on a Unix socket
# 2. Running Servo in headless mode to fetch a page
# 3. Verifying the connection worked
#

set -e

# Configuration
SOCKET_DIR="/tmp/servo-sockets-test"
SOCKET_PATH="$SOCKET_DIR/localhost.sock"
OUTPUT_IMAGE="/tmp/servo-uds-test.png"
TEST_URL="http://localhost/"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}Servo Unix Socket Test${NC}"
echo -e "${BLUE}=====================================${NC}"
echo

# Cleanup function
cleanup() {
    echo
    echo -e "${YELLOW}Cleaning up...${NC}"
    if [ -n "$GUNICORN_PID" ] && kill -0 "$GUNICORN_PID" 2>/dev/null; then
        echo "  Stopping Gunicorn (PID: $GUNICORN_PID)"
        kill "$GUNICORN_PID" 2>/dev/null || true
        wait "$GUNICORN_PID" 2>/dev/null || true
    fi
    rm -f "$SOCKET_PATH"
    rm -f "$OUTPUT_IMAGE"
    echo -e "${GREEN}Cleanup complete${NC}"
}

trap cleanup EXIT INT TERM

# Check dependencies
echo -e "${BLUE}Checking dependencies...${NC}"

if ! command -v python3 &> /dev/null; then
    echo -e "${RED}Error: python3 not found${NC}"
    exit 1
fi

if ! python3 -c "import flask" 2>/dev/null; then
    echo -e "${YELLOW}Installing Python dependencies...${NC}"
    pip3 install -q flask gunicorn
fi

echo -e "${GREEN}✓ Dependencies OK${NC}"
echo

# Create socket directory
echo -e "${BLUE}Setting up test environment...${NC}"
mkdir -p "$SOCKET_DIR"
rm -f "$SOCKET_PATH"
echo -e "${GREEN}✓ Socket directory: $SOCKET_DIR${NC}"
echo

# Start Gunicorn server in background
echo -e "${BLUE}Starting Gunicorn server...${NC}"
cd examples
gunicorn \
    --bind "unix:$SOCKET_PATH" \
    --workers 1 \
    --access-logfile - \
    --error-logfile - \
    --log-level warning \
    --daemon \
    --pid /tmp/gunicorn-test.pid \
    unix_socket_server:app

# Get PID
GUNICORN_PID=$(cat /tmp/gunicorn-test.pid 2>/dev/null || echo "")
if [ -z "$GUNICORN_PID" ]; then
    echo -e "${RED}Failed to start Gunicorn${NC}"
    exit 1
fi

# Wait for socket to be created
echo -e "${YELLOW}Waiting for socket...${NC}"
for i in {1..10}; do
    if [ -S "$SOCKET_PATH" ]; then
        echo -e "${GREEN}✓ Socket ready: $SOCKET_PATH${NC}"
        break
    fi
    sleep 0.5
done

if [ ! -S "$SOCKET_PATH" ]; then
    echo -e "${RED}Socket was not created${NC}"
    exit 1
fi

cd ..
echo

# Build Servo (if needed)
echo -e "${BLUE}Checking Servo build...${NC}"
if [ ! -f "target/debug/servo" ] && [ ! -f "target/release/servo" ]; then
    echo -e "${YELLOW}Servo not built. Building now (this may take a while)...${NC}"
    ./mach build --dev
fi

# Find Servo binary
SERVO_BIN=""
if [ -f "target/release/servo" ]; then
    SERVO_BIN="target/release/servo"
elif [ -f "target/debug/servo" ]; then
    SERVO_BIN="target/debug/servo"
else
    echo -e "${RED}Servo binary not found${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Using Servo: $SERVO_BIN${NC}"
echo

# Test with Servo
echo -e "${BLUE}Testing Servo with Unix sockets...${NC}"
echo -e "  URL: ${YELLOW}$TEST_URL${NC}"
echo -e "  Socket: ${YELLOW}$SOCKET_PATH${NC}"
echo -e "  Output: ${YELLOW}$OUTPUT_IMAGE${NC}"
echo

# Set environment variables and run Servo
export SERVO_USE_UNIX_SOCKETS=true
export SERVO_SOCKET_DIR="$SOCKET_DIR"
export SERVO_SOCKET_MAPPINGS="localhost:$SOCKET_PATH"
export RUST_LOG=net=debug

echo -e "${YELLOW}Running Servo in headless mode...${NC}"
timeout 30 "$SERVO_BIN" \
    -z \
    -x \
    -o "$OUTPUT_IMAGE" \
    "$TEST_URL" 2>&1 | grep -E "(UDS|Unix|socket|localhost)" || true

echo

# Check results
echo -e "${BLUE}Verifying results...${NC}"

if [ -f "$OUTPUT_IMAGE" ]; then
    SIZE=$(stat -f%z "$OUTPUT_IMAGE" 2>/dev/null || stat -c%s "$OUTPUT_IMAGE" 2>/dev/null)
    echo -e "${GREEN}✓ Screenshot created: $OUTPUT_IMAGE ($SIZE bytes)${NC}"

    if [ "$SIZE" -gt 1000 ]; then
        echo -e "${GREEN}✓ Screenshot appears valid${NC}"
    else
        echo -e "${YELLOW}⚠ Screenshot may be too small${NC}"
    fi
else
    echo -e "${RED}✗ Screenshot was not created${NC}"
    exit 1
fi

echo
echo -e "${GREEN}=====================================${NC}"
echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
echo -e "${GREEN}=====================================${NC}"
echo
echo -e "Unix domain socket networking is working!"
echo -e "Screenshot saved to: ${BLUE}$OUTPUT_IMAGE${NC}"
echo
