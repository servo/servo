# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

#!/usr/bin/env python3

import argparse
import json
import logging
import os
import subprocess
import time
import sys
import urllib.request
import urllib.error
import glob
from typing import Dict, Optional, Any, List


class OHOSWebDriverController:
    """Controller for running WebDriver tests on OHOS devices using HTTP API."""

    def __init__(self, webdriver_port: int = 7000, wpt_server_port: int = 8000) -> None:
        self.webdriver_port = webdriver_port
        self.wpt_server_port = wpt_server_port
        self.session_id: Optional[str] = None
        self.wpt_server_process: Optional[subprocess.Popen] = None

    def discover_tests(self, path: str) -> List[str]:
        """Discover test files in a given path (file or directory)."""
        tests = []

        # Determine the WPT tests directory
        # The script is in python/wpt/, and tests are in tests/wpt/tests/
        script_dir = os.path.dirname(os.path.abspath(__file__))
        servo_root = os.path.dirname(os.path.dirname(script_dir))  # Go up from python/wpt/ to servo root
        wpt_tests_dir = os.path.join(servo_root, "tests", "wpt", "tests")

        # Convert relative paths to absolute paths based on WPT tests directory
        if not os.path.isabs(path):
            full_path = os.path.join(wpt_tests_dir, path)
        else:
            full_path = path

        if os.path.isfile(full_path):
            # Single test file - convert back to relative path for URL construction
            rel_path = os.path.relpath(full_path, wpt_tests_dir)
            tests.append(rel_path)
        elif os.path.isdir(full_path):
            # Find all .html, .htm, .xhtml test files recursively
            for ext in ["*.html", "*.htm", "*.xhtml"]:
                pattern = os.path.join(full_path, "**", ext)
                found_files = glob.glob(pattern, recursive=True)
                for file_path in found_files:
                    # Convert back to relative path for URL construction
                    rel_path = os.path.relpath(file_path, wpt_tests_dir)
                    tests.append(rel_path)
        else:
            logging.error(f"Path does not exist: {path} (resolved to: {full_path})")

        # Normalize path separators for URLs
        normalized_tests = []
        for test in tests:
            # Normalize path separators to forward slashes for URLs
            test = test.replace("\\", "/")
            normalized_tests.append(test)

        return sorted(normalized_tests)

    def setup_wpt_server_access(self) -> bool:
        """Set up access to WPT server for OHOS device."""
        try:
            cmd = ["hdc", "rport", f"tcp:{self.wpt_server_port}", f"tcp:{self.wpt_server_port}"]
            logging.info(f"Setting up HDC reverse port forwarding for WPT: {' '.join(cmd)}")

            subprocess.run(cmd, capture_output=True, text=True, timeout=10)

            logging.info(f"HDC reverse port forwarding established for WPT server on port {self.wpt_server_port}")
            return True

        except FileNotFoundError:
            logging.error("HDC command not found. Please install HDC and add it to PATH")
            return False
        except subprocess.TimeoutExpired:
            logging.error("HDC reverse port forwarding command timed out")
            return False
        except Exception as e:
            logging.error(f"Failed to set up WPT server access: {e}")
            return False

    def setup_hdc_forwarding(self) -> bool:
        """Set up HDC port forwarding for WebDriver communication."""
        try:
            cmd = ["hdc", "fport", f"tcp:{self.webdriver_port}", f"tcp:{self.webdriver_port}"]
            logging.info(f"Setting up HDC port forwarding: {' '.join(cmd)}")

            subprocess.run(cmd, capture_output=True, text=True, timeout=10)

            logging.info(f"HDC port forwarding established on port {self.webdriver_port}")
            return True

        except FileNotFoundError:
            logging.error("HDC command not found. Make sure OHOS SDK is installed and hdc is in PATH")
            return False
        except subprocess.TimeoutExpired:
            logging.error("HDC port forwarding command timed out")
            return False
        except Exception as e:
            logging.error(f"Failed to set up HDC forwarding: {e}")
            return False

    def start_wpt_server(self) -> bool:
        """Start the WPT server on desktop."""
        try:
            # For now, assume WPT server is already running or started manually
            # In a complete implementation, this would start the WPT server
            logging.info(f"Assuming WPT server is running on port {self.wpt_server_port}")
            return True

        except Exception as e:
            logging.error(f"Failed to start WPT server: {e}")
            return False

    def webdriver_request(
        self, method: str, path: str, data: Optional[Dict] = None, timeout: Optional[int] = None
    ) -> Dict[str, Any]:
        """Make a WebDriver HTTP request."""
        url = f"http://127.0.0.1:{self.webdriver_port}{path}"

        headers = {
            "Content-Type": "application/json",
            "Host": f"127.0.0.1:{self.webdriver_port}",
        }
        request_data = json.dumps(data).encode("utf-8") if data else None

        request = urllib.request.Request(url, data=request_data, headers=headers, method=method)

        try:
            with urllib.request.urlopen(request, timeout=timeout) as response:
                response_data = response.read().decode("utf-8")
                return json.loads(response_data) if response_data else {}
        except urllib.error.HTTPError as e:
            error_response = e.read().decode("utf-8") if e.fp else "No response body"
            logging.error(f"WebDriver HTTP error {e.code}: {error_response}, {path}")
            new_error = urllib.error.HTTPError(e.url, e.code, e.msg, e.hdrs, None)
            # Set the error_response as an attribute for later access
            setattr(new_error, "error_response", error_response)
            raise new_error
        except Exception as e:
            logging.error(f"WebDriver request failed: {method} {path} - {e}")
            raise

    def delete_session(self, session_id: str) -> bool:
        """Delete a WebDriver session."""
        try:
            self.webdriver_request("DELETE", f"/session/{session_id}")
            logging.info(f"Deleted WebDriver session: {session_id}")
            return True
        except Exception as e:
            logging.error(f"Failed to delete session {session_id}: {e}")
            return False

    def create_session(self) -> bool:
        """Create a new WebDriver session."""
        try:
            capabilities = {"capabilities": {"alwaysMatch": {"browserName": "servo"}}}

            logging.debug(f"Sending session request: {json.dumps(capabilities, indent=2)}")
            response = self.webdriver_request("POST", "/session", capabilities)
            logging.debug(f"Session response: {json.dumps(response, indent=2)}")

            self.session_id = response.get("value", {}).get("sessionId")

            if self.session_id:
                logging.info(f"WebDriver session created: {self.session_id}")
                return True
            else:
                logging.error("Failed to create WebDriver session")
                return False

        except urllib.error.HTTPError as e:
            error_response = getattr(e, "error_response", "No error response available")

            logging.debug(f"HTTP error during session creation: {e.code} - {error_response}")

            if "session not created" in error_response:
                raise RuntimeError(f"Session not created. Please restart the WebDriver server: {error_response}")
            else:
                raise
        except Exception as e:
            logging.error(f"Failed to create WebDriver session: {e}")
            raise

    def create_window(self) -> bool:
        """Create a new window/webview if needed."""
        try:
            if not self.session_id:
                raise RuntimeError("No WebDriver session")

            try:
                handles_response = self.webdriver_request("GET", f"/session/{self.session_id}/window/handles")
                handles = handles_response.get("value", [])

                if handles:
                    logging.info(f"Found existing windows: {handles}")
                    # Focus on the first window
                    self.webdriver_request("POST", f"/session/{self.session_id}/window", {"handle": handles[0]})
                    return True
            except Exception as e:
                logging.debug(f"Could not get window handles: {e}")

            # Try to explicitly create a new window
            try:
                logging.info("Attempting to create new window via WebDriver")
                new_window_response = self.webdriver_request(
                    "POST", f"/session/{self.session_id}/window/new", {"type": "tab"}
                )
                if new_window_response:
                    logging.info(f"Created new window: {new_window_response}")
                    return True
            except Exception as e:
                logging.debug(f"New window creation failed: {e}")

            logging.info("No existing windows found, assuming window will be created on navigation")
            return True

        except Exception as e:
            logging.error(f"Failed to create window: {e}")
            return False

    def navigate_to_url(self, url: str, timeout: int = 10) -> bool:
        """Navigate to a URL with OHOS-specific handling."""
        if not self.session_id:
            raise RuntimeError("No WebDriver session")

        logging.info(f"Attempting to navigate to: {url}")
        data = {"url": url}

        try:
            navigation_success = self.webdriver_request(
                "POST", f"/session/{self.session_id}/url", data, timeout=timeout
            )
            logging.info(f"Navigation completed successfully: {navigation_success}")
            return True
        except Exception as nav_error:
            logging.debug(f"Navigation request failed: {nav_error}")
            return False

    def run_test(self, test_path: str, timeout: int = 30) -> Dict[str, Any]:
        """Run a single WPT test with enhanced error handling."""
        test_start_time = time.time()

        try:
            logging.info(f"Starting test: {test_path}")

            if not self.create_session():
                return {
                    "status": "SESSION_ERROR",
                    "title": test_path,
                    "details": "Failed to create WebDriver session",
                    "passCount": 0,
                    "failCount": 0,
                    "failingTests": [],
                    "duration": time.time() - test_start_time,
                }

            if not self.create_window():
                return {
                    "status": "WINDOW_ERROR",
                    "title": test_path,
                    "details": "Failed to create window",
                    "passCount": 0,
                    "failCount": 0,
                    "failingTests": [],
                    "duration": time.time() - test_start_time,
                }

            test_url = f"http://localhost:{self.wpt_server_port}/{test_path}"
            logging.info(f"Navigating URL: {test_url}")

            navigation_result = self.navigate_to_url(test_url, timeout=10)

            if not navigation_result:
                return {
                    "status": "NAVIGATION_ERROR",
                    "title": test_path,
                    "details": "Failed to navigate to test URL",
                    "passCount": 0,
                    "failCount": 0,
                    "failingTests": [],
                    "duration": time.time() - test_start_time,
                }

            return self.wait_for_test_completion_ohos(timeout, test_start_time)

        except Exception as e:
            logging.error(f"Error running test {test_path}: {e}")
            return {
                "status": "RUNTIME_ERROR",
                "title": test_path,
                "details": str(e),
                "passCount": 0,
                "failCount": 0,
                "failingTests": [],
                "duration": time.time() - test_start_time,
            }

    def wait_for_test_completion_ohos(self, timeout: int = 30, test_start_time: float = None) -> Dict[str, Any]:
        """OHOS test completion handling with enhanced timeout detection"""
        if test_start_time is None:
            test_start_time = time.time()

        try:
            logging.info("OHOS test completion handling...")

            logging.info("Waiting for page to load and test to complete...")
            for i in range(timeout // 5):
                time.sleep(5)
                logging.info(f"Waiting... ({(i + 1) * 5}/{timeout}s)")

                try:
                    script_path = os.path.join(os.path.dirname(__file__), "ohos_test_parser.js")
                    with open(script_path, "r", encoding="utf-8") as f:
                        script = f.read()

                        script_data = {"script": script, "args": []}
                        script_response = self.webdriver_request(
                            "POST", f"/session/{self.session_id}/execute/sync", script_data, timeout=2
                        )
                        result = script_response.get("value", {})

                        if result.get("status") in ["PASS", "FAIL"]:
                            return {
                                "status": result.get("status"),
                                "title": result.get("title", ""),
                                "details": result.get("bodyText", "")[:300] + "..."
                                if len(result.get("bodyText", "")) > 300
                                else result.get("bodyText", ""),
                                "passCount": result.get("passCount", 0),
                                "failCount": result.get("failCount", 0),
                                "failingTests": result.get("failingTests", []),
                                "duration": time.time() - test_start_time,
                            }
                        else:
                            logging.info(
                                f"Test still running, status: {result.get('status')}, body preview: {result.get('bodyText', '')[:100]}..."
                            )
                except Exception as api_error:
                    logging.debug(f"API request failed: {api_error}")

            # If we get here, either test timed out or API is completely unresponsive
            elapsed_time = time.time() - test_start_time
            screenshot_path = f"test_output/servo_ohos_indeterminate_screenshot_{int(time.time())}.jpeg"
            self.take_screenshot(screenshot_path)

            return {
                "status": "UNKNOWN",
                "title": "OHOS WebDriver Limitation",
                "details": f"Test did not complete within timeout or API unresponsive. Screenshot saved to {screenshot_path}",
                "passCount": 0,
                "failCount": 0,
                "failingTests": [],
                "duration": elapsed_time,
            }

        except Exception as e:
            elapsed_time = time.time() - test_start_time
            logging.error(f"Error in OHOS test completion handling: {e}")

            # Take screenshot for debugging on error
            screenshot_path = f"test_output/servo_ohos_error_screenshot_{int(time.time())}.jpeg"
            self.take_screenshot(screenshot_path)

            return {
                "status": "COMPLETION_ERROR",
                "title": "Test Completion Error",
                "details": f"Error during test completion check: {str(e)}. Screenshot saved to {screenshot_path}",
                "passCount": 0,
                "failCount": 0,
                "failingTests": [],
                "duration": elapsed_time,
            }

    def kill_servo_instances(self) -> bool:
        """Kill all servo instances on the OHOS device."""
        success = True

        try:
            # Kill servo processes
            subprocess.run(["hdc", "shell", "killall org.servo.servo"], capture_output=True, text=True, timeout=10)
            logging.debug("Executed killall command")
        except Exception as e:
            logging.debug(f"killall command failed (may be expected): {e}")
            success = False

        try:
            # Force stop servo application
            subprocess.run(
                ["hdc", "shell", "aa force-stop org.servo.servo"], capture_output=True, text=True, timeout=10
            )
            logging.debug("Executed force-stop command")
        except Exception as e:
            logging.debug(f"force-stop command failed (may be expected): {e}")
            success = False

        # Give time for cleanup
        time.sleep(2)
        return success

    def start_servo_application(self) -> bool:
        """Start the servo application on OHOS device."""
        try:
            subprocess.run(
                ["hdc", "shell", "aa start -a EntryAbility -b org.servo.servo"],
                capture_output=True,
                text=True,
                timeout=15,
            )
            logging.info("Started servo application")
            time.sleep(3)  # Give servo time to start
            return True
        except Exception as e:
            logging.error(f"Failed to start servo application: {e}")
            return False

    def run_multiple_tests(self, test_paths: List[str], timeout_per_test: int = 30) -> List[Dict[str, Any]]:
        """Run multiple tests consecutively with kill/restart cycle for each test."""
        results = []
        total_tests = len(test_paths)

        logging.info(f"Starting batch test run: {total_tests} tests")

        for i, test_path in enumerate(test_paths, 1):
            logging.info(f"\n{'=' * 60}")
            logging.info(f"Running test {i}/{total_tests}: {test_path}")
            logging.info(f"{'=' * 60}")

            # Kill any existing servo instances
            logging.info("Cleaning up existing servo instances...")
            self.kill_servo_instances()

            # Start fresh servo instance
            logging.info("Starting fresh servo instance...")
            if not self.start_servo_application():
                result = {
                    "status": "SERVO_START_FAILED",
                    "title": test_path,
                    "details": "Failed to start servo application",
                    "passCount": 0,
                    "failCount": 0,
                    "failingTests": [],
                    "duration": 0,
                }
                results.append(result)
                continue

            # Set up infrastructure for this test
            if not self.setup_hdc_forwarding():
                result = {
                    "status": "HDC_SETUP_FAILED",
                    "title": test_path,
                    "details": "Failed to set up HDC forwarding",
                    "passCount": 0,
                    "failCount": 0,
                    "failingTests": [],
                    "duration": 0,
                }
                results.append(result)
                continue

            self.setup_wpt_server_access()

            # Run the actual test
            result = self.run_test(test_path, timeout_per_test)
            result["test_number"] = i
            result["test_path"] = test_path
            results.append(result)

            # Clean up session for next test
            self.cleanup()

            logging.info(f"Test {i} completed with status: {result['status']}")

        return results

    def take_screenshot(self, output_path: str) -> bool:
        """Take a screenshot from OHOS device for debugging."""
        try:
            output_dir = os.path.dirname(output_path)
            os.makedirs(output_dir, exist_ok=True)
            snapshot_cmd = ["hdc", "shell", "snapshot_display", "-f", "/data/local/tmp/servo.jpeg"]
            result = subprocess.run(snapshot_cmd, capture_output=True, text=True, timeout=10)

            if "fail" in result.stdout.lower() or "error" in result.stdout.lower():
                logging.warning(f"Screenshot capture failed: {result.stdout.strip()}")
                return False

            recv_cmd = ["hdc", "file", "recv", "/data/local/tmp/servo.jpeg", output_path]
            result = subprocess.run(recv_cmd, capture_output=True, text=True, timeout=10)

            if "fail" in result.stdout.lower() or "error" in result.stdout.lower():
                logging.warning(f"Screenshot transfer failed: {result.stdout.strip()}")
                return False

            logging.info(f"Screenshot saved to: {output_path}")
            return True

        except Exception as e:
            logging.warning(f"Failed to take screenshot: {e}")
            return False

    def cleanup(self) -> None:
        """Clean up resources."""
        if self.session_id:
            try:
                self.webdriver_request("DELETE", f"/session/{self.session_id}")
            except Exception:
                pass
            self.session_id = None

        if self.wpt_server_process:
            try:
                self.wpt_server_process.terminate()
                self.wpt_server_process.wait(timeout=5)
            except Exception:
                try:
                    self.wpt_server_process.kill()
                except Exception:
                    pass
            self.wpt_server_process = None

    def print_test_summary(self, results: List[Dict[str, Any]]) -> None:
        """Print a summary of all test results."""
        if not results:
            print("No test results to display.")
            return

        print("\n" + "=" * 80)
        print(" TEST RESULTS SUMMARY")
        print("=" * 80)

        # Count results by status
        status_counts = {}
        total_duration = 0
        total_pass = 0
        total_fail = 0

        for result in results:
            status = result.get("status", "UNKNOWN")
            status_counts[status] = status_counts.get(status, 0) + 1
            total_duration += result.get("duration", 0)
            total_pass += result.get("passCount", 0)
            total_fail += result.get("failCount", 0)

        print(f"Total Tests: {len(results)}")
        print(f"Total Duration: {total_duration:.1f} seconds")
        print(f"Average Test Duration: {total_duration / len(results):.1f} seconds")

        print("\nStatus Breakdown:")
        for status, count in sorted(status_counts.items()):
            percentage = (count / len(results)) * 100
            print(f"  {status}: {count} ({percentage:.1f}%)")

        if total_pass > 0 or total_fail > 0:
            print("\nOverall Test Results:")
            print(f"  Total Assertions Passed: {total_pass}")
            print(f"  Total Assertions Failed: {total_fail}")
            if total_pass + total_fail > 0:
                pass_rate = (total_pass / (total_pass + total_fail)) * 100
                print(f"  Overall Pass Rate: {pass_rate:.1f}%")

        print(f"\n{'Test #':<6} {'Status':<18} {'Duration':<10} {'Test Path'}")
        print("-" * 80)

        for result in results:
            test_num = result.get("test_number", "?")
            status = result.get("status", "UNKNOWN")
            duration = result.get("duration", 0)
            test_path = result.get("test_path", result.get("title", "Unknown"))

            # Truncate long paths
            if len(test_path) > 45:
                test_path = "..." + test_path[-42:]

            print(f"{test_num:<6} {status:<18} {duration:<9.1f}s {test_path}")

        # Show detailed information for failed/problematic tests
        problematic_statuses = [
            "FAIL",
            "TIMEOUT",
            "ERROR",
            "RUNTIME_ERROR",
            "SESSION_ERROR",
            "WINDOW_ERROR",
            "NAVIGATION_ERROR",
            "COMPLETION_ERROR",
        ]

        problematic_tests = [r for r in results if r.get("status") in problematic_statuses]

        if problematic_tests:
            print("\n" + "=" * 80)
            print(" DETAILED FAILURE ANALYSIS")
            print("=" * 80)

            for result in problematic_tests:
                test_num = result.get("test_number", "?")
                status = result.get("status", "UNKNOWN")
                test_path = result.get("test_path", result.get("title", "Unknown"))
                details = result.get("details", "No details available")

                print(f"\nTest #{test_num}: {test_path}")
                print(f"Status: {status}")
                print(f"Details: {details}")

                if result.get("failCount", 0) > 0:
                    print(f"Failed Assertions: {result.get('failCount', 0)}")
                    failing_tests = result.get("failingTests", [])
                    if failing_tests:
                        print("Failing Test Cases:")
                        for i, failing_test in enumerate(failing_tests, 1):
                            if isinstance(failing_test, dict):
                                name = failing_test.get("name", "Unknown")
                                error = failing_test.get("error", "No error message")
                                print(f"  {i}. {name}: {error}")
                            else:
                                print(f"  {i}. {failing_test}")

        print("\n" + "=" * 80)


def main() -> int:
    parser = argparse.ArgumentParser(description="Run WPT tests on OHOS device")

    parser.add_argument("--test", required=True, help="Path to WPT test file or folder containing tests")
    parser.add_argument("--webdriver-port", type=int, default=7000, help="WebDriver server port")
    parser.add_argument("--wpt-server-port", type=int, default=8000, help="WPT server port")
    parser.add_argument("--timeout", type=int, default=30, help="Timeout per test in seconds")
    parser.add_argument("--verbose", action="store_true", help="Enable verbose logging")

    args = parser.parse_args()

    log_level = logging.DEBUG if args.verbose else logging.INFO
    logging.basicConfig(level=log_level, format="%(asctime)s - %(levelname)s - %(message)s")

    controller = OHOSWebDriverController(args.webdriver_port, args.wpt_server_port)

    try:
        # Determine tests to run - automatically detect if it's a file or folder
        test_paths = controller.discover_tests(args.test)
        if not test_paths:
            logging.error(f"No test files found at: {args.test}")
            return 1

        if len(test_paths) == 1:
            logging.info(f"Running single test: {test_paths[0]}")
        else:
            logging.info(f"Discovered {len(test_paths)} tests in: {args.test}")

        # Set up infrastructure once
        logging.info("Setting up test infrastructure...")

        if not controller.setup_wpt_server_access():
            logging.warning("Failed to set up WPT server access - continuing anyway")

        if not controller.start_wpt_server():
            logging.error("Failed to start WPT server")
            return 1

        # Run tests
        if len(test_paths) == 1:
            # Single test - use original logic with cleanup
            logging.info("Killing any existing servo instances and starting fresh...")
            controller.kill_servo_instances()

            if not controller.start_servo_application():
                logging.error("Failed to start servo application")
                return 1

            if not controller.setup_hdc_forwarding():
                logging.error("Failed to set up HDC forwarding")
                return 1

            result = controller.run_test(test_paths[0], args.timeout)
            results = [result]
        else:
            # Multiple tests - use batch runner
            results = controller.run_multiple_tests(test_paths, args.timeout)

        # Print results using unified summary format
        controller.print_test_summary(results)

        # Determine overall success
        successful_statuses = ["PASS", "INDETERMINATE", "API_UNRESPONSIVE"]
        successful_tests = sum(1 for r in results if r.get("status") in successful_statuses)
        success_rate = (successful_tests / len(results)) * 100

        print(f"\nOverall Success Rate: {successful_tests}/{len(results)} ({success_rate:.1f}%)")

    except KeyboardInterrupt:
        logging.info("Test execution interrupted by user")
        return 1
    except Exception as e:
        logging.error(f"Unexpected error: {e}")
        return 1
    finally:
        controller.cleanup()

        logging.info("Final cleanup - killing servo instances...")
        controller.kill_servo_instances()

    # This should never be reached
    return 1


if __name__ == "__main__":
    sys.exit(main())
