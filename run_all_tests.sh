#!/bin/bash
# Comprehensive Test Runner for Servo UDS Implementation
# Runs all test suites and reports results

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Log file
LOG_FILE="/tmp/servo-uds-tests-$(date +%Y%m%d-%H%M%S).log"

echo "=========================================="
echo "Servo UDS Comprehensive Test Suite"
echo "=========================================="
echo "Log file: $LOG_FILE"
echo

# Function to print section header
print_section() {
    echo
    echo -e "${BLUE}=========================================="
    echo "$1"
    echo "==========================================${NC}"
    echo
}

# Function to record test result
record_result() {
    local test_name="$1"
    local status="$2"
    local details="${3:-}"

    ((TOTAL_TESTS++))

    if [ "$status" = "PASS" ]; then
        ((PASSED_TESTS++))
        echo -e "${GREEN}✓ PASS${NC}: $test_name"
    elif [ "$status" = "FAIL" ]; then
        ((FAILED_TESTS++))
        echo -e "${RED}✗ FAIL${NC}: $test_name"
        if [ -n "$details" ]; then
            echo -e "  ${RED}$details${NC}"
        fi
    elif [ "$status" = "SKIP" ]; then
        ((SKIPPED_TESTS++))
        echo -e "${YELLOW}⊘ SKIP${NC}: $test_name"
        if [ -n "$details" ]; then
            echo -e "  ${YELLOW}$details${NC}"
        fi
    fi

    echo "$status: $test_name $details" >> "$LOG_FILE"
}

# Test 1: Rust Unit Tests
test_rust_units() {
    print_section "Test Suite 1: Rust Unit Tests"

    echo "Running transport_url tests..."
    if cargo test --lib -p net transport_url::tests 2>&1 | tee -a "$LOG_FILE"; then
        record_result "Rust unit tests (transport_url)" "PASS"
    else
        record_result "Rust unit tests (transport_url)" "FAIL"
    fi

    echo
    echo "Running unix_socket tests..."
    if cargo test --lib -p net unix_socket_tests 2>&1 | tee -a "$LOG_FILE"; then
        record_result "Rust unit tests (unix_socket)" "PASS"
    else
        record_result "Rust unit tests (unix_socket)" "FAIL"
    fi
}

# Test 2: Python Integration Tests
test_python_integration() {
    print_section "Test Suite 2: Python Integration Tests"

    if [ ! -f "test_uds_python.py" ]; then
        record_result "Python integration tests" "SKIP" "test_uds_python.py not found"
        return
    fi

    # Ensure Gunicorn is running
    echo "Starting Gunicorn for tests..."
    cd examples
    gunicorn --bind unix:/tmp/test.sock --workers 1 --daemon --pid /tmp/gunicorn-test.pid unix_socket_server:app 2>&1 | tee -a "$LOG_FILE"
    cd ..

    sleep 2

    # Check if socket exists
    if [ ! -S /tmp/test.sock ]; then
        record_result "Python integration tests" "FAIL" "Gunicorn socket not created"
        return
    fi

    # Run Python tests
    if python3 test_uds_python.py 2>&1 | tee -a "$LOG_FILE"; then
        record_result "Python integration tests (15 tests)" "PASS"
    else
        record_result "Python integration tests (15 tests)" "FAIL"
    fi

    # Cleanup
    if [ -f /tmp/gunicorn-test.pid ]; then
        kill "$(cat /tmp/gunicorn-test.pid)" 2>/dev/null || true
        rm -f /tmp/gunicorn-test.pid
    fi
    rm -f /tmp/test.sock
}

# Test 3: Bash Integration Tests
test_bash_integration() {
    print_section "Test Suite 3: Bash Integration Tests"

    if [ ! -f "test_uds_integration.sh" ]; then
        record_result "Bash integration tests" "SKIP" "test_uds_integration.sh not found"
        return
    fi

    if bash test_uds_integration.sh 2>&1 | tee -a "$LOG_FILE"; then
        record_result "Bash integration tests (curl-based)" "PASS"
    else
        record_result "Bash integration tests (curl-based)" "FAIL"
    fi
}

# Test 4: Rust Integration Tests
test_rust_integration() {
    print_section "Test Suite 4: Rust Integration Tests"

    if [ ! -f "tests/uds_integration_test.rs" ]; then
        record_result "Rust integration tests" "SKIP" "tests/uds_integration_test.rs not found"
        return
    fi

    # Ensure Gunicorn is running
    echo "Starting Gunicorn for Rust integration tests..."
    cd examples
    gunicorn --bind unix:/tmp/test.sock --workers 1 --daemon --pid /tmp/gunicorn-rust-test.pid unix_socket_server:app 2>&1 | tee -a "$LOG_FILE"
    cd ..

    sleep 2

    if cargo test --test uds_integration_test 2>&1 | tee -a "$LOG_FILE"; then
        record_result "Rust integration tests (hyperlocal)" "PASS"
    else
        record_result "Rust integration tests (hyperlocal)" "FAIL"
    fi

    # Cleanup
    if [ -f /tmp/gunicorn-rust-test.pid ]; then
        kill "$(cat /tmp/gunicorn-rust-test.pid)" 2>/dev/null || true
        rm -f /tmp/gunicorn-rust-test.pid
    fi
    rm -f /tmp/test.sock
}

# Test 5: UDS-Only Verification (CRITICAL!)
test_no_ip() {
    print_section "Test Suite 5: UDS-Only Verification (No IP Networking Allowed!)"

    echo "This test verifies NO IP-based networking:"
    echo "  - No TCP (IPv4/IPv6)"
    echo "  - No UDP (IPv4/IPv6)"
    echo "  - No SCTP"
    echo "  - No raw IP sockets"
    echo

    if [ ! -f "verify_uds_only.sh" ]; then
        record_result "UDS-only verification" "SKIP" "verify_uds_only.sh not found"
        return
    fi

    # Run comprehensive UDS-only verification
    if bash verify_uds_only.sh 2>&1 | tee -a "$LOG_FILE"; then
        record_result "UDS-only verification (13 protocol checks)" "PASS"
    else
        record_result "UDS-only verification (13 protocol checks)" "FAIL" "IP networking detected!"
    fi
}

# Test 6: Compilation Check
test_compilation() {
    print_section "Test Suite 6: Compilation Check"

    echo "Running cargo check..."
    if cargo check -p net 2>&1 | tee -a "$LOG_FILE"; then
        record_result "Cargo check (net package)" "PASS"
    else
        record_result "Cargo check (net package)" "FAIL"
    fi

    echo
    echo "Checking for warnings..."
    if cargo check -p net 2>&1 | grep -q "warning:"; then
        record_result "No compilation warnings" "FAIL" "Warnings found"
        cargo check -p net 2>&1 | grep "warning:" | tee -a "$LOG_FILE"
    else
        record_result "No compilation warnings" "PASS"
    fi
}

# Test 7: Code Formatting
test_formatting() {
    print_section "Test Suite 7: Code Formatting"

    echo "Checking Rust formatting..."
    if cargo fmt --all -- --check 2>&1 | tee -a "$LOG_FILE"; then
        record_result "Rust code formatting" "PASS"
    else
        record_result "Rust code formatting" "FAIL" "Run: cargo fmt --all"
    fi
}

# Print final summary
print_summary() {
    print_section "Test Summary"

    echo "Total tests run: $TOTAL_TESTS"
    echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"

    if [ $FAILED_TESTS -gt 0 ]; then
        echo -e "${RED}Failed: $FAILED_TESTS${NC}"
    else
        echo -e "Failed: $FAILED_TESTS"
    fi

    if [ $SKIPPED_TESTS -gt 0 ]; then
        echo -e "${YELLOW}Skipped: $SKIPPED_TESTS${NC}"
    else
        echo "Skipped: $SKIPPED_TESTS"
    fi

    echo
    echo "Success rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%"
    echo

    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}=========================================="
        echo "✓ ALL TESTS PASSED!"
        echo "==========================================${NC}"
        return 0
    else
        echo -e "${RED}=========================================="
        echo "✗ SOME TESTS FAILED"
        echo "==========================================${NC}"
        echo
        echo "See log file for details: $LOG_FILE"
        return 1
    fi
}

# Main test execution
main() {
    # Create log file
    echo "Servo UDS Test Run - $(date)" > "$LOG_FILE"
    echo "========================================" >> "$LOG_FILE"

    # Run all test suites
    test_compilation
    test_formatting
    test_rust_units
    test_python_integration
    test_bash_integration
    test_rust_integration
    test_no_ip  # CRITICAL!

    # Print summary
    print_summary
    exit $?
}

# Cleanup on exit
cleanup() {
    echo
    echo "Cleaning up..."

    # Kill any remaining Gunicorn processes
    pkill -f "gunicorn.*unix_socket_server" 2>/dev/null || true

    # Remove temporary sockets
    rm -f /tmp/test.sock /tmp/verify-test.sock
    rm -f /tmp/gunicorn-*.pid
}

trap cleanup EXIT

# Run tests
main
