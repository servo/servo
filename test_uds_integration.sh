#!/bin/bash
#
# Integration tests for Unix Domain Socket server
#
# Tests Gunicorn serving over UDS using curl
#

set -e

# Configuration
SOCKET_DIR="/tmp/servo-uds-test"
SOCKET_PATH="$SOCKET_DIR/test.sock"
TEST_PORT=18765  # Fallback TCP port for comparison

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Track test results
TESTS_PASSED=0
TESTS_FAILED=0

# Cleanup function
cleanup() {
    if [ -n "$GUNICORN_PID" ] && kill -0 "$GUNICORN_PID" 2>/dev/null; then
        echo -e "${YELLOW}Stopping Gunicorn (PID: $GUNICORN_PID)${NC}"
        kill "$GUNICORN_PID" 2>/dev/null || true
        wait "$GUNICORN_PID" 2>/dev/null || true
    fi
    rm -rf "$SOCKET_DIR"
}

trap cleanup EXIT INT TERM

# Test assertion helper
assert_equals() {
    local expected="$1"
    local actual="$2"
    local test_name="$3"

    if [ "$expected" = "$actual" ]; then
        echo -e "${GREEN}✓ PASS${NC}: $test_name"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC}: $test_name"
        echo -e "  Expected: $expected"
        echo -e "  Actual:   $actual"
        ((TESTS_FAILED++))
        return 1
    fi
}

assert_contains() {
    local haystack="$1"
    local needle="$2"
    local test_name="$3"

    if echo "$haystack" | grep -q "$needle"; then
        echo -e "${GREEN}✓ PASS${NC}: $test_name"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC}: $test_name"
        echo -e "  Expected to find: $needle"
        echo -e "  In: $haystack"
        ((TESTS_FAILED++))
        return 1
    fi
}

assert_success() {
    local exit_code="$1"
    local test_name="$2"

    if [ "$exit_code" -eq 0 ]; then
        echo -e "${GREEN}✓ PASS${NC}: $test_name"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC}: $test_name (exit code: $exit_code)"
        ((TESTS_FAILED++))
        return 1
    fi
}

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Unix Domain Socket Integration Tests${NC}"
echo -e "${BLUE}========================================${NC}"
echo

# Test 1: Check curl supports Unix sockets
echo -e "${BLUE}Test 1: Check curl supports --unix-socket${NC}"
if curl --version | grep -q "unix-sockets"; then
    echo -e "${GREEN}✓ PASS${NC}: curl supports Unix sockets"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}⚠ WARNING${NC}: curl may not support --unix-socket flag"
    echo "  This is not critical but some tests may fail"
fi
echo

# Test 2: Check Python dependencies
echo -e "${BLUE}Test 2: Check Python dependencies${NC}"
if python3 -c "import flask, gunicorn" 2>/dev/null; then
    echo -e "${GREEN}✓ PASS${NC}: Flask and Gunicorn are installed"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}Installing Python dependencies...${NC}"
    pip3 install -q flask gunicorn
    assert_success $? "Install Python dependencies"
fi
echo

# Test 3: Create socket directory
echo -e "${BLUE}Test 3: Create socket directory${NC}"
mkdir -p "$SOCKET_DIR"
assert_success $? "Create socket directory: $SOCKET_DIR"
echo

# Test 4: Start Gunicorn on Unix socket
echo -e "${BLUE}Test 4: Start Gunicorn server on Unix socket${NC}"
cd examples
gunicorn \
    --bind "unix:$SOCKET_PATH" \
    --workers 1 \
    --access-logfile - \
    --error-logfile - \
    --log-level warning \
    --daemon \
    --pid /tmp/gunicorn-integration-test.pid \
    unix_socket_server:app

GUNICORN_PID=$(cat /tmp/gunicorn-integration-test.pid 2>/dev/null)
cd ..

# Wait for socket to be created
sleep 2

if [ -S "$SOCKET_PATH" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Gunicorn started, socket created: $SOCKET_PATH"
    ((TESTS_PASSED++))
    echo -e "  PID: $GUNICORN_PID"
else
    echo -e "${RED}✗ FAIL${NC}: Socket file not created"
    ((TESTS_FAILED++))
    exit 1
fi
echo

# Test 5: Verify socket is listening
echo -e "${BLUE}Test 5: Verify socket is listening${NC}"
if [ -S "$SOCKET_PATH" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Socket exists and is a socket file"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ FAIL${NC}: Not a socket file"
    ((TESTS_FAILED++))
fi
echo

# Test 6: HTTP GET request to root path
echo -e "${BLUE}Test 6: HTTP GET request to / via Unix socket${NC}"
RESPONSE=$(curl -s --unix-socket "$SOCKET_PATH" http://localhost/)
assert_contains "$RESPONSE" "Hello from Unix Socket Server" "GET / returns HTML page"
echo

# Test 7: Check HTTP status code
echo -e "${BLUE}Test 7: HTTP status code for /${NC}"
STATUS=$(curl -s -w "%{http_code}" -o /dev/null --unix-socket "$SOCKET_PATH" http://localhost/)
assert_equals "200" "$STATUS" "HTTP status code is 200"
echo

# Test 8: API endpoint JSON response
echo -e "${BLUE}Test 8: API endpoint returns JSON${NC}"
API_RESPONSE=$(curl -s --unix-socket "$SOCKET_PATH" http://localhost/api/data)
assert_contains "$API_RESPONSE" '"transport": "unix_domain_socket"' "API response contains transport type"
assert_contains "$API_RESPONSE" '"server": "gunicorn + flask"' "API response contains server info"
echo

# Test 9: Test page endpoint
echo -e "${BLUE}Test 9: Test page endpoint${NC}"
TEST_RESPONSE=$(curl -s --unix-socket "$SOCKET_PATH" http://localhost/test)
assert_contains "$TEST_RESPONSE" "Test Page" "Test page loads"
assert_contains "$TEST_RESPONSE" "Successfully loaded via Unix socket" "Test page has correct content"
echo

# Test 10: Multiple concurrent requests
echo -e "${BLUE}Test 10: Multiple concurrent requests${NC}"
SUCCESS_COUNT=0
for i in {1..5}; do
    if curl -s --unix-socket "$SOCKET_PATH" http://localhost/ | grep -q "Hello from Unix Socket"; then
        ((SUCCESS_COUNT++))
    fi
done
assert_equals "5" "$SUCCESS_COUNT" "All 5 concurrent requests succeeded"
echo

# Test 11: HTTP headers
echo -e "${BLUE}Test 11: Check HTTP headers${NC}"
HEADERS=$(curl -sI --unix-socket "$SOCKET_PATH" http://localhost/)
assert_contains "$HEADERS" "Content-Type" "Response has Content-Type header"
assert_contains "$HEADERS" "HTTP/1.1 200" "Response has HTTP/1.1 200 status line"
echo

# Test 12: POST request
echo -e "${BLUE}Test 12: POST request to API${NC}"
POST_RESPONSE=$(curl -s --unix-socket "$SOCKET_PATH" \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"test": "data"}' \
    http://localhost/api/data)
assert_contains "$POST_RESPONSE" "unix_domain_socket" "POST request works"
echo

# Test 13: Large response handling
echo -e "${BLUE}Test 13: Handle larger response${NC}"
LARGE_RESPONSE=$(curl -s --unix-socket "$SOCKET_PATH" http://localhost/)
RESPONSE_SIZE=${#LARGE_RESPONSE}
if [ "$RESPONSE_SIZE" -gt 500 ]; then
    echo -e "${GREEN}✓ PASS${NC}: Large response handled (${RESPONSE_SIZE} bytes)"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ FAIL${NC}: Response too small (${RESPONSE_SIZE} bytes)"
    ((TESTS_FAILED++))
fi
echo

# Test 14: Error handling - 404
echo -e "${BLUE}Test 14: 404 Error handling${NC}"
STATUS_404=$(curl -s -w "%{http_code}" -o /dev/null --unix-socket "$SOCKET_PATH" http://localhost/nonexistent)
assert_equals "404" "$STATUS_404" "404 status for nonexistent page"
echo

# Test 15: Socket permissions
echo -e "${BLUE}Test 15: Socket file permissions${NC}"
if [ -r "$SOCKET_PATH" ] && [ -w "$SOCKET_PATH" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Socket is readable and writable"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ FAIL${NC}: Socket permissions issue"
    ((TESTS_FAILED++))
fi
echo

# Summary
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Test Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "Total tests run: $((TESTS_PASSED + TESTS_FAILED))"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo

if [ "$TESTS_FAILED" -eq 0 ]; then
    echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
    exit 0
else
    echo -e "${RED}✗ SOME TESTS FAILED${NC}"
    exit 1
fi
