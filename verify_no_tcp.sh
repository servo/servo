#!/bin/bash
# TCP Verification Script - Ensures NO TCP connections for UDS-only operation
# This is CRITICAL - the whole point is NO TCP, not even loopback!

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SOCKET_PATH="/tmp/verify-tcp-test.sock"
PID_FILE="/tmp/verify-tcp-gunicorn.pid"
REPORT_FILE="/tmp/servo-tcp-verification-$(date +%Y%m%d-%H%M%S).txt"

echo "=========================================="
echo "TCP Verification for Servo UDS"
echo "=========================================="
echo "This script verifies that NO TCP connections"
echo "are made when using Unix domain sockets."
echo
echo "Report file: $REPORT_FILE"
echo

# Initialize report
cat > "$REPORT_FILE" << EOF
TCP Verification Report
Generated: $(date)
========================================

Testing Scenario:
- Start Gunicorn on Unix socket ONLY
- Make HTTP requests via Unix socket
- Verify NO TCP listeners
- Verify NO TCP connections

EOF

# Cleanup function
cleanup() {
    echo
    echo "Cleaning up..."

    if [ -f "$PID_FILE" ]; then
        kill "$(cat $PID_FILE)" 2>/dev/null || true
        rm -f "$PID_FILE"
    fi

    rm -f "$SOCKET_PATH"
}

trap cleanup EXIT

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check dependencies
echo -e "${BLUE}Checking required tools...${NC}"

if ! command_exists ss; then
    echo -e "${RED}✗ 'ss' command not found${NC}"
    echo "  Install with: sudo apt-get install iproute2"
    exit 1
fi

if ! command_exists lsof; then
    echo -e "${YELLOW}⚠ 'lsof' not found (optional)${NC}"
fi

if ! command_exists netstat; then
    echo -e "${YELLOW}⚠ 'netstat' not found (optional)${NC}"
fi

echo -e "${GREEN}✓ Required tools available${NC}"
echo

# Start Gunicorn on Unix socket ONLY
echo -e "${BLUE}Starting Gunicorn on Unix socket...${NC}"

if [ ! -f "examples/unix_socket_server.py" ]; then
    echo -e "${RED}✗ Flask app not found: examples/unix_socket_server.py${NC}"
    exit 1
fi

cd examples
gunicorn \
    --bind "unix:$SOCKET_PATH" \
    --workers 1 \
    --daemon \
    --pid "$PID_FILE" \
    unix_socket_server:app
cd ..

# Wait for socket
sleep 2

if [ ! -S "$SOCKET_PATH" ]; then
    echo -e "${RED}✗ Unix socket not created${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Gunicorn started${NC}"
echo

# Get Gunicorn PID
GUNICORN_PID=$(cat "$PID_FILE")
echo "Gunicorn PID: $GUNICORN_PID"
echo

# Verification Tests
TESTS_PASSED=0
TESTS_FAILED=0

# Test 1: Check for TCP listeners on standard ports
echo -e "${BLUE}Test 1: Checking for TCP listeners on standard HTTP ports...${NC}"
echo "Test 1: TCP Listeners on Standard Ports" >> "$REPORT_FILE"

TCP_LISTENERS=$(ss -tlnp 2>/dev/null | grep -E ':(80|443|8000|8080|8443|5000)' || true)

if [ -n "$TCP_LISTENERS" ]; then
    echo -e "${RED}✗ FAIL: Found TCP listeners on standard ports${NC}"
    echo "$TCP_LISTENERS"
    echo "FAIL: Found TCP listeners" >> "$REPORT_FILE"
    echo "$TCP_LISTENERS" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: No TCP listeners on standard ports${NC}"
    echo "PASS: No TCP listeners on standard ports" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 2: Check for ANY TCP listeners from Gunicorn
echo -e "${BLUE}Test 2: Checking for ANY TCP listeners from Gunicorn process...${NC}"
echo "Test 2: TCP Listeners from Gunicorn" >> "$REPORT_FILE"

GUNICORN_TCP_LISTENERS=$(ss -tlnp 2>/dev/null | grep gunicorn || true)

if [ -n "$GUNICORN_TCP_LISTENERS" ]; then
    echo -e "${RED}✗ FAIL: Gunicorn has TCP listeners!${NC}"
    echo "$GUNICORN_TCP_LISTENERS"
    echo "FAIL: Gunicorn is listening on TCP" >> "$REPORT_FILE"
    echo "$GUNICORN_TCP_LISTENERS" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: Gunicorn has no TCP listeners${NC}"
    echo "PASS: Gunicorn has no TCP listeners" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 3: Check for TCP connections from Gunicorn
echo -e "${BLUE}Test 3: Checking for TCP connections from Gunicorn...${NC}"
echo "Test 3: TCP Connections from Gunicorn" >> "$REPORT_FILE"

GUNICORN_TCP_CONNECTIONS=$(ss -tnp 2>/dev/null | grep gunicorn || true)

if [ -n "$GUNICORN_TCP_CONNECTIONS" ]; then
    echo -e "${RED}✗ FAIL: Gunicorn has active TCP connections!${NC}"
    echo "$GUNICORN_TCP_CONNECTIONS"
    echo "FAIL: Gunicorn has TCP connections" >> "$REPORT_FILE"
    echo "$GUNICORN_TCP_CONNECTIONS" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: Gunicorn has no TCP connections${NC}"
    echo "PASS: No TCP connections" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 4: Check for loopback TCP connections (127.0.0.1)
echo -e "${BLUE}Test 4: Checking for loopback TCP connections (127.0.0.1)...${NC}"
echo "Test 4: Loopback TCP Connections" >> "$REPORT_FILE"

LOOPBACK_CONNECTIONS=$(ss -tnp 2>/dev/null | grep "127.0.0.1" | grep gunicorn || true)

if [ -n "$LOOPBACK_CONNECTIONS" ]; then
    echo -e "${RED}✗ FAIL: Found loopback TCP connections!${NC}"
    echo "$LOOPBACK_CONNECTIONS"
    echo "FAIL: Loopback TCP connections found" >> "$REPORT_FILE"
    echo "$LOOPBACK_CONNECTIONS" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: No loopback TCP connections${NC}"
    echo "PASS: No loopback connections" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 5: Verify Unix socket is being used
echo -e "${BLUE}Test 5: Verifying Unix socket is active...${NC}"
echo "Test 5: Unix Socket Verification" >> "$REPORT_FILE"

if [ -S "$SOCKET_PATH" ]; then
    echo -e "${GREEN}✓ PASS: Unix socket exists: $SOCKET_PATH${NC}"
    echo "PASS: Unix socket exists at $SOCKET_PATH" >> "$REPORT_FILE"
    ((TESTS_PASSED++))

    # Check if Gunicorn is using it
    if command_exists lsof; then
        if lsof "$SOCKET_PATH" 2>/dev/null | grep -q gunicorn; then
            echo -e "${GREEN}✓ PASS: Gunicorn is using Unix socket${NC}"
            echo "PASS: Gunicorn has the socket open" >> "$REPORT_FILE"
        else
            echo -e "${YELLOW}⚠ WARNING: Cannot verify socket ownership (may need sudo)${NC}"
            echo "WARNING: Could not verify socket ownership" >> "$REPORT_FILE"
        fi
    fi
else
    echo -e "${RED}✗ FAIL: Unix socket does not exist${NC}"
    echo "FAIL: No Unix socket found" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 6: Make request and verify no TCP during request
echo -e "${BLUE}Test 6: Making HTTP request and monitoring TCP activity...${NC}"
echo "Test 6: TCP Activity During Request" >> "$REPORT_FILE"

# Capture TCP state before request
TCP_BEFORE=$(ss -tn 2>/dev/null | grep -c "ESTAB" || echo "0")

# Make request via Unix socket
curl -s --unix-socket "$SOCKET_PATH" http://localhost/ > /dev/null 2>&1

# Capture TCP state after request
TCP_AFTER=$(ss -tn 2>/dev/null | grep -c "ESTAB" || echo "0")

# Check if Gunicorn created any TCP connections
GUNICORN_TCP_AFTER=$(ss -tnp 2>/dev/null | grep gunicorn || true)

if [ -n "$GUNICORN_TCP_AFTER" ]; then
    echo -e "${RED}✗ FAIL: TCP connections appeared after request!${NC}"
    echo "$GUNICORN_TCP_AFTER"
    echo "FAIL: TCP activity detected during request" >> "$REPORT_FILE"
    echo "$GUNICORN_TCP_AFTER" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: No TCP connections during request${NC}"
    echo "PASS: No TCP activity during Unix socket request" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 7: Check file descriptors
if command_exists lsof; then
    echo -e "${BLUE}Test 7: Checking open file descriptors...${NC}"
    echo "Test 7: Open File Descriptors" >> "$REPORT_FILE"

    # Get all FDs for Gunicorn
    echo "Open files for PID $GUNICORN_PID:" >> "$REPORT_FILE"
    lsof -p "$GUNICORN_PID" 2>/dev/null | grep -E "unix|sock|TCP|UDP" >> "$REPORT_FILE" || true

    # Check for TCP sockets
    if lsof -p "$GUNICORN_PID" 2>/dev/null | grep -q "TCP"; then
        echo -e "${RED}✗ FAIL: Found TCP sockets in file descriptors${NC}"
        lsof -p "$GUNICORN_PID" | grep "TCP"
        echo "FAIL: TCP sockets found in file descriptors" >> "$REPORT_FILE"
        ((TESTS_FAILED++))
    else
        echo -e "${GREEN}✓ PASS: No TCP sockets in file descriptors${NC}"
        echo "PASS: No TCP sockets in file descriptors" >> "$REPORT_FILE"
        ((TESTS_PASSED++))
    fi
    echo >> "$REPORT_FILE"
    echo
fi

# Test 8: Network namespace check (if available)
if command_exists ip; then
    echo -e "${BLUE}Test 8: Checking network namespaces...${NC}"
    echo "Test 8: Network Namespace" >> "$REPORT_FILE"

    NS_INFO=$(ip netns identify $GUNICORN_PID 2>/dev/null || echo "default")
    echo "Process network namespace: $NS_INFO" | tee -a "$REPORT_FILE"
    echo >> "$REPORT_FILE"
    echo
fi

# Final Summary
echo >> "$REPORT_FILE"
echo "========================================" >> "$REPORT_FILE"
echo "SUMMARY" >> "$REPORT_FILE"
echo "========================================" >> "$REPORT_FILE"
echo "Total tests: $((TESTS_PASSED + TESTS_FAILED))" >> "$REPORT_FILE"
echo "Passed: $TESTS_PASSED" >> "$REPORT_FILE"
echo "Failed: $TESTS_FAILED" >> "$REPORT_FILE"
echo >> "$REPORT_FILE"

echo "=========================================="
echo "VERIFICATION SUMMARY"
echo "=========================================="
echo
echo "Total tests: $((TESTS_PASSED + TESTS_FAILED))"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"

if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}Failed: $TESTS_FAILED${NC}"
    echo
    echo -e "${RED}=========================================="
    echo "✗ TCP VERIFICATION FAILED!"
    echo "==========================================${NC}"
    echo
    echo "TCP connections were detected. This violates"
    echo "the UDS-only requirement."
    echo
    echo "See report: $REPORT_FILE"
    echo >> "$REPORT_FILE"
    echo "RESULT: FAILED - TCP detected" >> "$REPORT_FILE"
    exit 1
else
    echo -e "Failed: $TESTS_FAILED"
    echo
    echo -e "${GREEN}=========================================="
    echo "✓ TCP VERIFICATION PASSED!"
    echo "==========================================${NC}"
    echo
    echo "No TCP connections detected."
    echo "System is operating in UDS-only mode."
    echo
    echo "Full report: $REPORT_FILE"
    echo >> "$REPORT_FILE"
    echo "RESULT: PASSED - No TCP detected" >> "$REPORT_FILE"
    exit 0
fi
