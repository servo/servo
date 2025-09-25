#!/usr/bin/env python3

"""
Test script to verify crash vs timeout detection logic
"""

import os
import sys
import time
from unittest.mock import Mock, MagicMock

# Add the directory containing ohos_webdriver_test to the path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from ohos_webdriver_test import OHOSWebDriverController


class MockWebDriverController(OHOSWebDriverController):
    """Mock controller for testing crash/timeout detection"""

    def __init__(self):
        # Skip the parent initialization to avoid setting up real connections
        self.webdriver_port = 7000
        self.wpt_server_port = 8000
        self.session_id = "mock-session-id"
        self.wpt_server_process = None

    def take_screenshot(self, output_path: str) -> bool:
        """Mock screenshot function"""
        print(f"Mock: Taking screenshot at {output_path}")
        return True

    def webdriver_request(self, method: str, path: str, data=None, timeout=None):
        """Mock WebDriver request that can simulate different failure patterns"""
        # This will be overridden in test cases
        pass


def test_crash_detection():
    """Test that consecutive WebDriver failures are detected as crashes"""
    print("Testing Crash Detection")
    print("=" * 40)

    controller = MockWebDriverController()

    # Mock webdriver_request to always fail (simulating a crash)
    def failing_request(*args, **kwargs):
        raise Exception("WebDriver communication failed")

    controller.webdriver_request = failing_request

    # Mock the script file reading
    import tempfile

    with tempfile.NamedTemporaryFile(mode="w", suffix=".js", delete=False) as f:
        f.write('console.log("mock script");')
        mock_script_path = f.name

    # Patch the script path
    original_join = os.path.join

    def mock_join(*args):
        if args[-1] == "ohos_test_parser.js":
            return mock_script_path
        return original_join(*args)

    os.path.join = mock_join

    try:
        start_time = time.time()
        result = controller.wait_for_test_completion_ohos(timeout=5, test_start_time=start_time)

        print(f"Result status: {result['status']}")
        print(f"Result title: {result['title']}")
        print(f"Duration: {result['duration']:.1f}s")

        # Should detect crash after 3 consecutive failures
        assert result["status"] == "CRASH", f"Expected CRASH, got {result['status']}"
        assert result["duration"] < 5, f"Should crash quickly, took {result['duration']:.1f}s"

        print("✓ Crash detection working correctly")

    finally:
        os.path.join = original_join
        os.unlink(mock_script_path)


def test_timeout_detection():
    """Test that long-running tests are detected as timeouts"""
    print("\nTesting Timeout Detection")
    print("=" * 40)

    controller = MockWebDriverController()

    # Mock webdriver_request to return "still running" status
    def timeout_request(*args, **kwargs):
        return {
            "value": {
                "status": "RUNNING",  # Not PASS or FAIL
                "title": "Test Still Running",
                "bodyText": "Test is still executing...",
            }
        }

    controller.webdriver_request = timeout_request

    # Mock the script file reading
    import tempfile

    with tempfile.NamedTemporaryFile(mode="w", suffix=".js", delete=False) as f:
        f.write('console.log("mock script");')
        mock_script_path = f.name

    # Patch the script path
    original_join = os.path.join

    def mock_join(*args):
        if args[-1] == "ohos_test_parser.js":
            return mock_script_path
        return original_join(*args)

    os.path.join = mock_join

    try:
        start_time = time.time()
        result = controller.wait_for_test_completion_ohos(timeout=3, test_start_time=start_time)

        print(f"Result status: {result['status']}")
        print(f"Result title: {result['title']}")
        print(f"Duration: {result['duration']:.1f}s")

        # Should detect timeout after waiting full duration
        assert result["status"] == "TIMEOUT", f"Expected TIMEOUT, got {result['status']}"
        assert result["duration"] >= 3, f"Should wait full timeout, took {result['duration']:.1f}s"

        print("✓ Timeout detection working correctly")

    finally:
        os.path.join = original_join
        os.unlink(mock_script_path)


def test_successful_completion():
    """Test that successful completion is still detected correctly"""
    print("\nTesting Successful Completion")
    print("=" * 40)

    controller = MockWebDriverController()

    call_count = 0

    def success_request(*args, **kwargs):
        nonlocal call_count
        call_count += 1

        if call_count <= 2:
            # First couple calls - test still running
            return {"value": {"status": "RUNNING", "title": "Test Running", "bodyText": "Test is executing..."}}
        else:
            # Third call - test completed
            return {
                "value": {
                    "status": "PASS",
                    "title": "Test Passed",
                    "bodyText": "All assertions passed",
                    "passCount": 5,
                    "failCount": 0,
                    "failingTests": [],
                }
            }

    controller.webdriver_request = success_request

    # Mock the script file reading
    import tempfile

    with tempfile.NamedTemporaryFile(mode="w", suffix=".js", delete=False) as f:
        f.write('console.log("mock script");')
        mock_script_path = f.name

    # Patch the script path
    original_join = os.path.join

    def mock_join(*args):
        if args[-1] == "ohos_test_parser.js":
            return mock_script_path
        return original_join(*args)

    os.path.join = mock_join

    try:
        start_time = time.time()
        result = controller.wait_for_test_completion_ohos(timeout=10, test_start_time=start_time)

        print(f"Result status: {result['status']}")
        print(f"Result title: {result['title']}")
        print(f"Pass count: {result['passCount']}")
        print(f"Duration: {result['duration']:.1f}s")

        # Should complete successfully
        assert result["status"] == "PASS", f"Expected PASS, got {result['status']}"
        assert result["passCount"] == 5, f"Expected 5 passes, got {result['passCount']}"
        assert result["duration"] < 10, f"Should complete quickly, took {result['duration']:.1f}s"

        print("✓ Successful completion working correctly")

    finally:
        os.path.join = original_join
        os.unlink(mock_script_path)


def main():
    print("Testing OHOS WebDriver Crash/Timeout Detection")
    print("=" * 50)

    try:
        test_crash_detection()
        test_timeout_detection()
        test_successful_completion()

        print("\n" + "=" * 50)
        print("✅ All tests passed!")
        print("Crash vs Timeout detection is working correctly")

    except Exception as e:
        print(f"\n❌ Test failed: {e}")
        import traceback

        traceback.print_exc()
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
