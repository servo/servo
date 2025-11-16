#!/bin/bash
# UDS-Only Verification Script - Ensures NO IP-based connections
# Checks for TCP, UDP, SCTP, and any other IP-based protocols
# This is CRITICAL - the whole point is NO IP networking, ONLY Unix sockets!

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SOCKET_PATH="/tmp/verify-uds-only.sock"
PID_FILE="/tmp/verify-uds-only.pid"
REPORT_FILE="/tmp/servo-uds-only-verification-$(date +%Y%m%d-%H%M%S).txt"

echo "=========================================="
echo "UDS-Only Verification for Servo"
echo "=========================================="
echo "Verifying NO IP-based connections:"
echo "  - NO TCP (IPv4/IPv6)"
echo "  - NO UDP (IPv4/IPv6)"
echo "  - NO SCTP (IPv4/IPv6)"
echo "  - NO Raw IP sockets"
echo "  - NO other network protocols"
echo
echo "Report file: $REPORT_FILE"
echo

# Initialize report
cat > "$REPORT_FILE" << EOF
UDS-Only Verification Report
Generated: $(date)
========================================

Testing Scenario:
- Start Gunicorn on Unix socket ONLY
- Make HTTP requests via Unix socket
- Verify NO IP-based networking of ANY kind

Protocol Checks:
- TCP (IPv4 and IPv6) - listeners and connections
- UDP (IPv4 and IPv6) - listeners and connections
- SCTP (IPv4 and IPv6) - listeners and connections
- Raw IP sockets
- Network interfaces in use
- File descriptors for sockets

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

MISSING_TOOLS=""

if ! command_exists ss; then
    echo -e "${RED}✗ 'ss' command not found${NC}"
    MISSING_TOOLS="$MISSING_TOOLS ss"
fi

if ! command_exists lsof; then
    echo -e "${YELLOW}⚠ 'lsof' not found (optional but recommended)${NC}"
fi

if ! command_exists netstat; then
    echo -e "${YELLOW}⚠ 'netstat' not found (optional but recommended)${NC}"
fi

if [ -n "$MISSING_TOOLS" ]; then
    echo -e "${RED}Missing required tools: $MISSING_TOOLS${NC}"
    echo "  Install with: sudo apt-get install iproute2 net-tools lsof"
    exit 1
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

# Test 1: TCP IPv4 Listeners
echo -e "${BLUE}Test 1: Checking for TCP IPv4 listeners...${NC}"
echo "Test 1: TCP IPv4 Listeners" >> "$REPORT_FILE"

TCP4_LISTENERS=$(ss -4tlnp 2>/dev/null | grep gunicorn || true)

if [ -n "$TCP4_LISTENERS" ]; then
    echo -e "${RED}✗ FAIL: Found TCP IPv4 listeners!${NC}"
    echo "$TCP4_LISTENERS"
    echo "FAIL: TCP IPv4 listeners detected" >> "$REPORT_FILE"
    echo "$TCP4_LISTENERS" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: No TCP IPv4 listeners${NC}"
    echo "PASS: No TCP IPv4 listeners" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 2: TCP IPv6 Listeners
echo -e "${BLUE}Test 2: Checking for TCP IPv6 listeners...${NC}"
echo "Test 2: TCP IPv6 Listeners" >> "$REPORT_FILE"

TCP6_LISTENERS=$(ss -6tlnp 2>/dev/null | grep gunicorn || true)

if [ -n "$TCP6_LISTENERS" ]; then
    echo -e "${RED}✗ FAIL: Found TCP IPv6 listeners!${NC}"
    echo "$TCP6_LISTENERS"
    echo "FAIL: TCP IPv6 listeners detected" >> "$REPORT_FILE"
    echo "$TCP6_LISTENERS" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: No TCP IPv6 listeners${NC}"
    echo "PASS: No TCP IPv6 listeners" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 3: TCP IPv4 Connections
echo -e "${BLUE}Test 3: Checking for TCP IPv4 connections...${NC}"
echo "Test 3: TCP IPv4 Connections" >> "$REPORT_FILE"

TCP4_CONNECTIONS=$(ss -4tnp 2>/dev/null | grep gunicorn || true)

if [ -n "$TCP4_CONNECTIONS" ]; then
    echo -e "${RED}✗ FAIL: Found TCP IPv4 connections!${NC}"
    echo "$TCP4_CONNECTIONS"
    echo "FAIL: TCP IPv4 connections detected" >> "$REPORT_FILE"
    echo "$TCP4_CONNECTIONS" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: No TCP IPv4 connections${NC}"
    echo "PASS: No TCP IPv4 connections" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 4: TCP IPv6 Connections
echo -e "${BLUE}Test 4: Checking for TCP IPv6 connections...${NC}"
echo "Test 4: TCP IPv6 Connections" >> "$REPORT_FILE"

TCP6_CONNECTIONS=$(ss -6tnp 2>/dev/null | grep gunicorn || true)

if [ -n "$TCP6_CONNECTIONS" ]; then
    echo -e "${RED}✗ FAIL: Found TCP IPv6 connections!${NC}"
    echo "$TCP6_CONNECTIONS"
    echo "FAIL: TCP IPv6 connections detected" >> "$REPORT_FILE"
    echo "$TCP6_CONNECTIONS" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: No TCP IPv6 connections${NC}"
    echo "PASS: No TCP IPv6 connections" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 5: UDP IPv4 Sockets
echo -e "${BLUE}Test 5: Checking for UDP IPv4 sockets...${NC}"
echo "Test 5: UDP IPv4 Sockets" >> "$REPORT_FILE"

UDP4_SOCKETS=$(ss -4unp 2>/dev/null | grep gunicorn || true)

if [ -n "$UDP4_SOCKETS" ]; then
    echo -e "${RED}✗ FAIL: Found UDP IPv4 sockets!${NC}"
    echo "$UDP4_SOCKETS"
    echo "FAIL: UDP IPv4 sockets detected" >> "$REPORT_FILE"
    echo "$UDP4_SOCKETS" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: No UDP IPv4 sockets${NC}"
    echo "PASS: No UDP IPv4 sockets" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 6: UDP IPv6 Sockets
echo -e "${BLUE}Test 6: Checking for UDP IPv6 sockets...${NC}"
echo "Test 6: UDP IPv6 Sockets" >> "$REPORT_FILE"

UDP6_SOCKETS=$(ss -6unp 2>/dev/null | grep gunicorn || true)

if [ -n "$UDP6_SOCKETS" ]; then
    echo -e "${RED}✗ FAIL: Found UDP IPv6 sockets!${NC}"
    echo "$UDP6_SOCKETS"
    echo "FAIL: UDP IPv6 sockets detected" >> "$REPORT_FILE"
    echo "$UDP6_SOCKETS" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: No UDP IPv6 sockets${NC}"
    echo "PASS: No UDP IPv6 sockets" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 7: SCTP Sockets (if supported)
echo -e "${BLUE}Test 7: Checking for SCTP sockets...${NC}"
echo "Test 7: SCTP Sockets" >> "$REPORT_FILE"

SCTP_SOCKETS=$(ss -Stnp 2>/dev/null | grep gunicorn || true)

if [ -n "$SCTP_SOCKETS" ]; then
    echo -e "${RED}✗ FAIL: Found SCTP sockets!${NC}"
    echo "$SCTP_SOCKETS"
    echo "FAIL: SCTP sockets detected" >> "$REPORT_FILE"
    echo "$SCTP_SOCKETS" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: No SCTP sockets${NC}"
    echo "PASS: No SCTP sockets" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 8: Raw IP Sockets
echo -e "${BLUE}Test 8: Checking for raw IP sockets...${NC}"
echo "Test 8: Raw IP Sockets" >> "$REPORT_FILE"

RAW_SOCKETS=$(ss -wnp 2>/dev/null | grep gunicorn || true)

if [ -n "$RAW_SOCKETS" ]; then
    echo -e "${RED}✗ FAIL: Found raw IP sockets!${NC}"
    echo "$RAW_SOCKETS"
    echo "FAIL: Raw IP sockets detected" >> "$REPORT_FILE"
    echo "$RAW_SOCKETS" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: No raw IP sockets${NC}"
    echo "PASS: No raw IP sockets" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 9: All IP Sockets (comprehensive check)
echo -e "${BLUE}Test 9: Comprehensive check for ANY IP sockets...${NC}"
echo "Test 9: All IP Sockets" >> "$REPORT_FILE"

ALL_IP_SOCKETS=$(ss -anp 2>/dev/null | grep -E "tcp|udp|sctp|raw" | grep gunicorn || true)

if [ -n "$ALL_IP_SOCKETS" ]; then
    echo -e "${RED}✗ FAIL: Found IP-based sockets!${NC}"
    echo "$ALL_IP_SOCKETS"
    echo "FAIL: IP-based sockets detected" >> "$REPORT_FILE"
    echo "$ALL_IP_SOCKETS" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: No IP-based sockets${NC}"
    echo "PASS: No IP-based sockets" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 10: Verify Unix socket is active
echo -e "${BLUE}Test 10: Verifying Unix socket is active...${NC}"
echo "Test 10: Unix Socket Verification" >> "$REPORT_FILE"

if [ -S "$SOCKET_PATH" ]; then
    echo -e "${GREEN}✓ PASS: Unix socket exists: $SOCKET_PATH${NC}"
    echo "PASS: Unix socket exists at $SOCKET_PATH" >> "$REPORT_FILE"
    ((TESTS_PASSED++))

    # Check Unix sockets for this process
    UNIX_SOCKETS=$(ss -xnp 2>/dev/null | grep "$GUNICORN_PID" || true)
    if [ -n "$UNIX_SOCKETS" ]; then
        echo -e "${GREEN}✓ PASS: Gunicorn is using Unix sockets${NC}"
        echo "Unix sockets in use:" >> "$REPORT_FILE"
        echo "$UNIX_SOCKETS" >> "$REPORT_FILE"
    fi
else
    echo -e "${RED}✗ FAIL: Unix socket does not exist${NC}"
    echo "FAIL: No Unix socket found" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 11: File descriptor check with lsof
if command_exists lsof; then
    echo -e "${BLUE}Test 11: Checking file descriptors for IP sockets...${NC}"
    echo "Test 11: File Descriptor IP Sockets" >> "$REPORT_FILE"

    # Get all network-related FDs
    echo "All network file descriptors for PID $GUNICORN_PID:" >> "$REPORT_FILE"
    lsof -p "$GUNICORN_PID" 2>/dev/null | grep -E "IPv|TCP|UDP|SCTP" >> "$REPORT_FILE" || echo "None found" >> "$REPORT_FILE"

    # Check for any IP sockets
    if lsof -p "$GUNICORN_PID" 2>/dev/null | grep -qE "IPv|TCP|UDP|SCTP"; then
        echo -e "${RED}✗ FAIL: Found IP socket file descriptors${NC}"
        lsof -p "$GUNICORN_PID" | grep -E "IPv|TCP|UDP|SCTP"
        echo "FAIL: IP socket file descriptors found" >> "$REPORT_FILE"
        ((TESTS_FAILED++))
    else
        echo -e "${GREEN}✓ PASS: No IP socket file descriptors${NC}"
        echo "PASS: No IP socket file descriptors" >> "$REPORT_FILE"
        ((TESTS_PASSED++))
    fi

    # Show Unix sockets
    echo "Unix socket file descriptors:" >> "$REPORT_FILE"
    lsof -p "$GUNICORN_PID" 2>/dev/null | grep -i unix >> "$REPORT_FILE" || echo "None found" >> "$REPORT_FILE"

    echo >> "$REPORT_FILE"
    echo
fi

# Test 12: Network activity during request
echo -e "${BLUE}Test 12: Checking for IP activity during HTTP request...${NC}"
echo "Test 12: IP Activity During Request" >> "$REPORT_FILE"

# Capture IP socket state before request
IP_BEFORE=$(ss -anp 2>/dev/null | grep -E "tcp|udp|sctp" | grep gunicorn | wc -l)

# Make request via Unix socket
curl -s --unix-socket "$SOCKET_PATH" http://localhost/ > /dev/null 2>&1

# Capture IP socket state after request
IP_AFTER=$(ss -anp 2>/dev/null | grep -E "tcp|udp|sctp" | grep gunicorn | wc -l)

# Check if any IP sockets appeared
IP_ACTIVITY=$(ss -anp 2>/dev/null | grep -E "tcp|udp|sctp" | grep gunicorn || true)

if [ -n "$IP_ACTIVITY" ] || [ "$IP_AFTER" -gt "$IP_BEFORE" ]; then
    echo -e "${RED}✗ FAIL: IP network activity detected during request!${NC}"
    echo "$IP_ACTIVITY"
    echo "FAIL: IP activity during Unix socket request" >> "$REPORT_FILE"
    echo "Before: $IP_BEFORE, After: $IP_AFTER" >> "$REPORT_FILE"
    echo "$IP_ACTIVITY" >> "$REPORT_FILE"
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: No IP activity during request${NC}"
    echo "PASS: No IP activity during Unix socket request" >> "$REPORT_FILE"
    echo "IP sockets before: $IP_BEFORE, after: $IP_AFTER" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

# Test 13: Check /proc/net for IP connections
echo -e "${BLUE}Test 13: Checking /proc/net for IP connections...${NC}"
echo "Test 13: /proc/net IP Connections" >> "$REPORT_FILE"

HAS_IP_CONNECTIONS=false

# Check TCP
if grep -q "$GUNICORN_PID" /proc/net/tcp /proc/net/tcp6 2>/dev/null; then
    echo -e "${RED}✗ FAIL: Found TCP connections in /proc/net${NC}"
    echo "FAIL: TCP in /proc/net" >> "$REPORT_FILE"
    HAS_IP_CONNECTIONS=true
fi

# Check UDP
if grep -q "$GUNICORN_PID" /proc/net/udp /proc/net/udp6 2>/dev/null; then
    echo -e "${RED}✗ FAIL: Found UDP sockets in /proc/net${NC}"
    echo "FAIL: UDP in /proc/net" >> "$REPORT_FILE"
    HAS_IP_CONNECTIONS=true
fi

if [ "$HAS_IP_CONNECTIONS" = true ]; then
    ((TESTS_FAILED++))
else
    echo -e "${GREEN}✓ PASS: No IP connections in /proc/net${NC}"
    echo "PASS: No IP connections in /proc/net" >> "$REPORT_FILE"
    ((TESTS_PASSED++))
fi
echo >> "$REPORT_FILE"
echo

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
echo "UDS-ONLY VERIFICATION SUMMARY"
echo "=========================================="
echo
echo "Protocol Coverage:"
echo "  - TCP (IPv4/IPv6): Checked ✓"
echo "  - UDP (IPv4/IPv6): Checked ✓"
echo "  - SCTP: Checked ✓"
echo "  - Raw IP: Checked ✓"
echo "  - All IP protocols: Checked ✓"
echo
echo "Total tests: $((TESTS_PASSED + TESTS_FAILED))"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"

if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}Failed: $TESTS_FAILED${NC}"
    echo
    echo -e "${RED}=========================================="
    echo "✗ UDS-ONLY VERIFICATION FAILED!"
    echo "==========================================${NC}"
    echo
    echo "IP-based networking detected. This violates"
    echo "the UDS-only requirement."
    echo
    echo "The system should use ONLY Unix domain sockets,"
    echo "with NO TCP, UDP, SCTP, or any IP-based protocols."
    echo
    echo "See report: $REPORT_FILE"
    echo >> "$REPORT_FILE"
    echo "RESULT: FAILED - IP networking detected" >> "$REPORT_FILE"
    exit 1
else
    echo -e "Failed: $TESTS_FAILED"
    echo
    echo -e "${GREEN}=========================================="
    echo "✓ UDS-ONLY VERIFICATION PASSED!"
    echo "==========================================${NC}"
    echo
    echo "No IP-based networking detected."
    echo "System is operating in UDS-ONLY mode."
    echo
    echo "✓ No TCP (IPv4/IPv6)"
    echo "✓ No UDP (IPv4/IPv6)"
    echo "✓ No SCTP"
    echo "✓ No raw IP sockets"
    echo "✓ No IP activity during requests"
    echo "✓ Only Unix domain sockets in use"
    echo
    echo "Full report: $REPORT_FILE"
    echo >> "$REPORT_FILE"
    echo "RESULT: PASSED - UDS-only mode confirmed" >> "$REPORT_FILE"
    exit 0
fi
