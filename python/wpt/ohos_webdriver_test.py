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
from configparser import ConfigParser
from typing import Dict, Optional, Any, List


class OHOSWebDriverController:
    """Controller for running WebDriver tests on OHOS devices using HTTP API."""

    def __init__(self, webdriver_port: int = 7000, wpt_server_port: int = 8000) -> None:
        self.webdriver_port = webdriver_port
        self.wpt_server_port = wpt_server_port
        self.session_id: Optional[str] = None
        self.wpt_server_process: Optional[subprocess.Popen] = None

        # Get path to WPT metadata directory
        script_dir = os.path.dirname(os.path.abspath(__file__))
        servo_root = os.path.dirname(os.path.dirname(script_dir))
        self.metadata_path = os.path.join(servo_root, "tests", "wpt", "meta")

    def discover_tests(self, path: str) -> List[str]:
        """Discover test files in a given path (file or directory)."""
        tests = []

        # The script is in python/wpt/, and tests are in tests/wpt/tests/
        script_dir = os.path.dirname(os.path.abspath(__file__))
        servo_root = os.path.dirname(os.path.dirname(script_dir))  # Go up from python/wpt/ to servo root
        wpt_tests_dir = os.path.join(servo_root, "tests", "wpt", "tests")

        # Convert relative paths to absolute paths based
        if not os.path.isabs(path):
            full_path = os.path.join(wpt_tests_dir, path)
        else:
            full_path = path

        if os.path.isfile(full_path):
            # Single test file - convert back to relative path for URL construction
            rel_path = os.path.relpath(full_path, wpt_tests_dir)
            tests.append(rel_path)
        elif os.path.isdir(full_path):
            # Find all .html, .htm, .xhtml test files
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
            test = test.replace("\\", "/")
            normalized_tests.append(test)

        return sorted(normalized_tests)

    def get_expected_path(self, test_path: str) -> str:
        """Get path to the expectation .ini file for a given test."""
        # Convert test path to .ini file path
        ini_path = test_path.replace("/", os.sep) + ".ini"
        return os.path.join(self.metadata_path, ini_path)

    def load_test_expectations(self, test_path: str) -> Dict[str, Any]:
        """Load test expectations from .ini file if it exists."""
        expectations = {
            "test_expected": "PASS",  # Default expectation for the test itself
            "subtest_expectations": {},  # Expected results for subtests
        }

        expected_file = self.get_expected_path(test_path)

        if not os.path.exists(expected_file):
            logging.debug(f"No expectations file found for {test_path} at {expected_file}")
            return expectations

        try:
            parser = ConfigParser()
            parser.read(expected_file, encoding="utf-8")

            # Get the test file section name (usually the filename)
            test_name = os.path.basename(test_path)

            # Load subtest expectations first
            for section_name in parser.sections():
                if section_name != test_name:
                    # This is a subtest section
                    subtest_section = parser[section_name]
                    if "expected" in subtest_section:
                        expectations["subtest_expectations"][section_name] = subtest_section["expected"]

            # Check for overall test expectation
            if parser.has_section(test_name):
                test_section = parser[test_name]
                if "expected" in test_section:
                    expectations["test_expected"] = test_section["expected"]
                else:
                    # If no explicit test-level expectation but we have subtest expectations,
                    # the test is generally expected to FAIL because some subtests are failing
                    if expectations["subtest_expectations"]:
                        # Check if any subtest is expected to fail
                        has_failing_subtests = any(
                            status in ["FAIL", "TIMEOUT", "ERROR"]
                            for status in expectations["subtest_expectations"].values()
                        )
                        if has_failing_subtests:
                            expectations["test_expected"] = "FAIL"
            else:
                # No test-level section, but if we have subtest expectations with failures,
                # the overall test should be expected to fail
                if expectations["subtest_expectations"]:
                    has_failing_subtests = any(
                        status in ["FAIL", "TIMEOUT", "ERROR"]
                        for status in expectations["subtest_expectations"].values()
                    )
                    if has_failing_subtests:
                        expectations["test_expected"] = "FAIL"

            logging.debug(f"Loaded expectations for {test_path}: {expectations}")

        except Exception as e:
            logging.warning(f"Failed to parse expectations file {expected_file}: {e}")

        return expectations

    def compare_with_expectations(self, test_path: str, result: Dict[str, Any]) -> Dict[str, Any]:
        """Compare test results with expectations and return analysis."""
        expectations = self.load_test_expectations(test_path)

        analysis = {
            "test_path": test_path,
            "has_expectations": os.path.exists(self.get_expected_path(test_path)),
            "test_expected": expectations["test_expected"],
            "test_actual": result.get("status", "UNKNOWN"),
            "test_matches_expectation": False,
            "unexpected_subtests": [],
            "expected_failures": [],
            "unexpected_passes": [],
            "summary": "UNKNOWN",
        }

        # Compare overall test result
        actual_status = result.get("status", "UNKNOWN")
        expected_status = expectations["test_expected"]

        # Map our result statuses to WPT statuses
        status_mapping = {
            "PASS": "PASS",
            "FAIL": "FAIL",
            "TIMEOUT": "TIMEOUT",
            "CRASH": "CRASH",
            "ERROR": "ERROR",
            "UNKNOWN": "FAIL",  # Treat unknown as fail for comparison
            # Legacy status handling
            "COMPLETION_ERROR": "CRASH",  # Map old completion errors to crash
            "TIMEOUT OR CRASH": "TIMEOUT",  # Default ambiguous cases to timeout
        }

        mapped_actual = status_mapping.get(actual_status, "FAIL")
        analysis["test_matches_expectation"] = mapped_actual == expected_status

        # If we have subtest expectations, analyze them
        failing_tests = result.get("failingTests", [])

        # For each failing subtest, check if it was expected to fail
        for failing_test in failing_tests:
            raw_subtest_name = failing_test.get("name", "")

            # Clean up the subtest name by removing common error patterns that might be appended
            subtest_name = raw_subtest_name

            # Try to find a match in expectations, considering that the parsed name might have extra content
            matched_expectation = None
            expected_subtest_status = None

            # First try exact match
            if subtest_name in expectations["subtest_expectations"]:
                matched_expectation = subtest_name
                expected_subtest_status = expectations["subtest_expectations"][subtest_name]
            else:
                # Try normalized matching with better name handling
                normalized_subtest = self._normalize_subtest_name(subtest_name)

                for expected_name in expectations["subtest_expectations"]:
                    normalized_expected = self._normalize_expectation_name(expected_name)

                    # Try various matching strategies
                    if (
                        # Exact match after normalization
                        normalized_expected == normalized_subtest
                        # Substring matching (in case of truncation)
                        or normalized_expected in normalized_subtest
                        or normalized_subtest in normalized_expected
                        # Try with original names too
                        or expected_name in subtest_name
                        or subtest_name in expected_name
                    ):
                        matched_expectation = expected_name
                        expected_subtest_status = expectations["subtest_expectations"][expected_name]
                        logging.debug(
                            f"Matched subtest '{subtest_name}' (normalized: '{normalized_subtest}') with expectation '{expected_name}' (normalized: '{normalized_expected}')"
                        )
                        break

            if matched_expectation and expected_subtest_status:
                if expected_subtest_status == "FAIL":
                    analysis["expected_failures"].append(
                        {
                            "name": subtest_name,
                            "error": failing_test.get("error", ""),
                            "matched_expectation": matched_expectation,
                        }
                    )
                else:
                    analysis["unexpected_subtests"].append(
                        {
                            "name": subtest_name,
                            "expected": expected_subtest_status,
                            "actual": "FAIL",
                            "error": failing_test.get("error", ""),
                            "matched_expectation": matched_expectation,
                        }
                    )
            else:
                # No expectation means it should pass
                analysis["unexpected_subtests"].append(
                    {"name": subtest_name, "expected": "PASS", "actual": "FAIL", "error": failing_test.get("error", "")}
                )

        # Check for unexpected passes (subtests that passed but were expected to fail)
        # This is harder to detect with our current result format, but we can infer it
        # if we know the total number of subtests from expectations vs actual results

        # Determine overall summary
        has_unexpected_subtests = len(analysis["unexpected_subtests"]) > 0

        # Categorize results into 5 specific categories based on expected vs actual outcome
        if analysis["test_matches_expectation"] and not has_unexpected_subtests:
            # Test result matches expectation and no unexpected subtest issues
            analysis["summary"] = "EXPECTED"
        elif not analysis["test_matches_expectation"]:
            # Test level expectation mismatch - categorize by actual result type
            if mapped_actual == "CRASH":
                analysis["summary"] = "UNEXPECTED_CRASH"
            elif mapped_actual == "TIMEOUT":
                analysis["summary"] = "UNEXPECTED_TIMEOUT"
            elif mapped_actual == "PASS" and expected_status != "PASS":
                analysis["summary"] = "UNEXPECTED_PASS"
            else:
                # FAIL or other non-pass results when expecting PASS (or different non-pass)
                analysis["summary"] = "UNEXPECTED_FAIL"
        elif has_unexpected_subtests:
            # Test level matches but subtests have issues - categorize by dominant pattern
            # For now, classify subtest issues as UNEXPECTED_FAIL since subtests are failing unexpectedly
            analysis["summary"] = "UNEXPECTED_FAIL"
        else:
            # Fallback - should be rare
            analysis["summary"] = "EXPECTED"

        return analysis

    def _normalize_subtest_name(self, name: str) -> str:
        """Normalize subtest names for better matching by removing common variations."""
        import re

        # Remove common assertion error patterns and location info
        patterns_to_remove = [
            r"assert_[a-zA-Z_]+:.*$",  # Remove assertion errors
            r"@http://[^:]+:\d+:\d+.*$",  # Remove line number references
            r"TypeError:.*$",  # Remove TypeError messages
            r"ReferenceError:.*$",  # Remove ReferenceError messages
            r"\s*expected\s+[A-Z]+,\s*got\s+[A-Z]+.*$",  # Remove expected/got messages
            r"runTests/<.*$",  # Remove runTests/< patterns
            r"must scroll.*$",  # Remove scroll-related assertion details
        ]

        normalized = name.strip()
        for pattern in patterns_to_remove:
            normalized = re.sub(pattern, "", normalized, flags=re.IGNORECASE)

        # Clean up any extra whitespace
        normalized = " ".join(normalized.split())

        return normalized

    def _normalize_expectation_name(self, name: str) -> str:
        """Normalize expectation names from INI files for better matching."""
        # Handle escaped backslashes in INI files (e.g., [Box B\] -> [Box B])
        normalized = name.replace("\\]", "]").replace("\\[", "[")

        # Remove any other escaping
        normalized = normalized.replace("\\", "")

        # Clean up whitespace
        normalized = " ".join(normalized.split())

        return normalized

    def setup_hdc_reverse_forwarding(self) -> bool:
        """Setting up HDC reverse port forwarding for WPT"""
        try:
            cmd = ["hdc", "rport", f"tcp:{self.wpt_server_port}", f"tcp:{self.wpt_server_port}"]
            logging.debug(f"Setting up HDC reverse forwarding: {' '.join(cmd)}")
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)

            if result.returncode != 0:
                logging.warning(f"HDC reverse forwarding failed with code {result.returncode}")
                logging.warning(f"stdout: {result.stdout}")
                logging.warning(f"stderr: {result.stderr}")
                return False

            logging.debug(f"HDC reverse forwarding successful: {result.stdout.strip()}")
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
            logging.debug(f"Setting up HDC forwarding: {' '.join(cmd)}")
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)

            if result.returncode != 0:
                logging.error(f"HDC forwarding failed with code {result.returncode}")
                logging.error(f"stdout: {result.stdout}")
                logging.error(f"stderr: {result.stderr}")
                return False

            logging.debug(f"HDC forwarding successful: {result.stdout.strip()}")
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
            # Determine the servo root directory and WPT tests path
            script_dir = os.path.dirname(os.path.abspath(__file__))
            servo_root = os.path.dirname(os.path.dirname(script_dir))  # Go up from python/wpt/ to servo root
            wpt_tests_dir = os.path.join(servo_root, "tests", "wpt", "tests")

            if not os.path.exists(wpt_tests_dir):
                logging.error(f"WPT tests directory not found: {wpt_tests_dir}")
                return False

            # Start WPT server using python wpt serve command
            cmd = [
                sys.executable,
                "wpt",
                "serve",
            ]

            logging.info(f"Starting WPT server: {' '.join(cmd)}")
            logging.info(f"Working directory: {wpt_tests_dir}")

            # Start the server process in the background
            self.wpt_server_process = subprocess.Popen(
                cmd, cwd=wpt_tests_dir, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
            )

            # Give the server a moment to start
            time.sleep(3)

            # Check if the process is running
            if self.wpt_server_process.poll() is not None:
                stdout, stderr = self.wpt_server_process.communicate()
                logging.error(f"WPT server failed to start. Exit code: {self.wpt_server_process.returncode}")
                logging.error(f"Stdout: {stdout}")
                logging.error(f"Stderr: {stderr}")
                return False

            logging.info(f"WPT server started successfully on port {self.wpt_server_port}")
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

    def delete_session(self, session_id: str, timeout: int = 5) -> bool:
        """Delete a WebDriver session."""
        try:
            self.webdriver_request("DELETE", f"/session/{session_id}", timeout=timeout)
            logging.info(f"Deleted WebDriver session: {session_id}")
            return True
        except Exception as e:
            logging.error(f"Failed to delete session {session_id}: {e}")
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

    def wait_for_test_completion_ohos(self, timeout: int = 15, test_start_time: float | None = None) -> Dict[str, Any]:
        """OHOS test completion handling"""

        if test_start_time is None:
            test_start_time = time.time()

        script_path = os.path.join(os.path.dirname(__file__), "ohos_test_parser.js")
        with open(script_path, "r", encoding="utf-8") as f:
            script = f.read()

        consecutive_failures = 0
        max_consecutive_failures = 3  # If we fail to communicate 3 times in a row, consider it a crash

        logging.info("Waiting for page to load and test to complete...")

        for i in range(timeout):
            logging.info(f"Waiting... ({i}/{timeout}s)")

            try:
                script_data = {"script": script, "args": []}
                script_response = self.webdriver_request(
                    "POST", f"/session/{self.session_id}/execute/sync", script_data, timeout=2
                )
                result = script_response.get("value", {})

                # Reset consecutive failures if we get a successful response
                consecutive_failures = 0

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
                    logging.info("Test still running")

            except Exception as api_error:
                consecutive_failures += 1
                logging.debug(f"API request failed: {api_error} (consecutive failures: {consecutive_failures})")

                # If we've had multiple consecutive failures, likely a crash
                if consecutive_failures >= max_consecutive_failures:
                    elapsed_time = time.time() - test_start_time
                    logging.error(
                        f"Detected crash: {consecutive_failures} consecutive WebDriver communication failures"
                    )

                    # Take screenshot for debugging
                    screenshot_path = f"test_output/servo_ohos_crash_screenshot_{int(time.time())}.jpeg"
                    self.take_screenshot(screenshot_path)

                    return {
                        "status": "CRASH",
                        "title": "Test Crashed",
                        "details": f"Browser/WebDriver crashed after {consecutive_failures} consecutive communication failures. Screenshot saved to {screenshot_path}",
                        "passCount": 0,
                        "failCount": 0,
                        "failingTests": [],
                        "duration": elapsed_time,
                    }

            if i != timeout - 1:
                time.sleep(1)

        # If we get here, test timed out but we can still communicate with WebDriver
        elapsed_time = time.time() - test_start_time
        screenshot_path = f"test_output/servo_ohos_timeout_screenshot_{int(time.time())}.jpeg"
        self.take_screenshot(screenshot_path)

        # Try one final check to see if test completed during our timeout handling
        try:
            script_data = {"script": script, "args": []}
            script_response = self.webdriver_request(
                "POST", f"/session/{self.session_id}/execute/sync", script_data, timeout=2
            )
            result = script_response.get("value", {})

            if result.get("status") in ["PASS", "FAIL"]:
                # Test actually completed just as we were timing out
                return {
                    "status": result.get("status"),
                    "title": result.get("title", ""),
                    "details": result.get("bodyText", "")[:300] + "..."
                    if len(result.get("bodyText", "")) > 300
                    else result.get("bodyText", ""),
                    "passCount": result.get("passCount", 0),
                    "failCount": result.get("failCount", 0),
                    "failingTests": result.get("failingTests", []),
                    "duration": elapsed_time,
                }
            else:
                # Test is still running but took too long - genuine timeout
                return {
                    "status": "TIMEOUT",
                    "title": result.get("title", "Test Timeout"),
                    "details": f"Test timed out after {timeout} seconds. Test was still running but exceeded time limit. Screenshot saved to {screenshot_path}",
                    "passCount": 0,
                    "failCount": 0,
                    "failingTests": [],
                    "duration": elapsed_time,
                }

        except Exception as final_error:
            # Final check failed too - this is likely a crash that happened during timeout
            logging.error(f"Final status check failed: {final_error}")
            return {
                "status": "CRASH",
                "title": "Test Crashed During Timeout",
                "details": f"WebDriver communication failed during timeout handling. Likely crashed. Screenshot saved to {screenshot_path}",
                "passCount": 0,
                "failCount": 0,
                "failingTests": [],
                "duration": elapsed_time,
            }

    def kill_servo_instances(self) -> bool:
        """Kill all servo instances on the OHOS device."""
        success = True

        try:
            subprocess.run(["hdc", "shell", "killall org.servo.servo"], capture_output=True, text=True, timeout=10)
        except Exception as e:
            logging.debug(f"killall command failed (may be expected): {e}")
            success = False

        return success

    def create_session(self) -> bool:
        """Create a new WebDriver session."""
        try:
            capabilities = {"capabilities": {"alwaysMatch": {"browserName": "servo"}}}
            response = self.webdriver_request("POST", "/session", capabilities)

            self.session_id = response.get("value", {}).get("sessionId")

            if self.session_id:
                return True
            else:
                logging.error("Failed to create WebDriver session - no session ID in response")
                return False
        except urllib.error.HTTPError as e:
            error_response = getattr(e, "error_response", "No error response available")

            logging.debug(f"HTTP error during session creation: {e.code} - {error_response}")

            if "session not created" in error_response:
                raise RuntimeError(f"Session not created. Please restart the WebDriver server: {error_response}")
            else:
                raise
        except urllib.error.URLError as e:
            logging.error(f"WebDriver connection error: {e}")
            raise
        except Exception as e:
            logging.error(f"Failed to create WebDriver session: {e}")
            raise

    def check_device_connection(self) -> bool:
        """Check if OHOS device is connected via HDC."""
        try:
            result = subprocess.run(["hdc", "list", "targets"], capture_output=True, text=True, timeout=10)
            if result.returncode != 0:
                logging.error("Failed to list HDC targets")
                return False

            targets = result.stdout.strip()
            if not targets or "[Empty]" in targets:
                logging.error("No OHOS devices connected. Please connect a device and try again.")
                return False

            logging.debug(f"Connected devices: {targets}")
            return True

        except FileNotFoundError:
            logging.error("HDC command not found. Please install OHOS SDK and add hdc to PATH")
            return False
        except Exception as e:
            logging.error(f"Failed to check device connection: {e}")
            return False

    def start_servo_application(self, test_path: str) -> bool:
        """Start the servo application on OHOS device."""
        try:
            # Check device connection first
            if not self.check_device_connection():
                return False

            # Start servo with the test URL
            start_cmd = [
                "hdc",
                "shell",
                f"aa start -a EntryAbility -b org.servo.servo -U http://localhost:{self.wpt_server_port}/{test_path}",
            ]
            logging.debug(f"Starting servo: {' '.join(start_cmd)}")

            result = subprocess.run(start_cmd, capture_output=True, text=True, timeout=15)

            if result.returncode != 0:
                logging.error(f"Failed to start servo application. Return code: {result.returncode}")
                logging.error(f"stdout: {result.stdout}")
                logging.error(f"stderr: {result.stderr}")
                return False

            logging.debug(f"Servo start result: {result.stdout.strip()}")

            time.sleep(0.5)

            # Try to create WebDriver session
            logging.debug(f"Attempting to create WebDriver session on port {self.webdriver_port}")
            if not self.create_session():
                logging.error("Failed to create WebDriver session after starting servo")
                return False

            logging.info(f"Successfully started servo and created WebDriver session: {self.session_id}")
            return True

        except Exception as e:
            logging.error(f"Failed to start servo application: {e}")
            return False

    def run_tests(self, test_paths: List[str], timeout_per_test: int = 15) -> List[Dict[str, Any]]:
        """Run tests consecutively with kill/restart cycle for each test."""
        results = []
        total_tests = len(test_paths)

        logging.info(f"Starting test run: {total_tests} tests")

        for i, test_path in enumerate(test_paths, 1):
            logging.info(f"\n{'=' * 60}")
            logging.info(f"Running test {i}/{total_tests}: {test_path}")
            logging.info(f"{'=' * 60}")
            test_start_time = time.time()

            # Kill any existing servo instances
            logging.info("Cleaning up existing servo instances...")
            self.kill_servo_instances()

            # Set up infrastructure BEFORE starting servo application
            logging.info("Setting up HDC port forwarding...")
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

            self.setup_hdc_reverse_forwarding()

            # Start fresh servo instance (after port forwarding is set up)
            logging.info("Starting fresh servo instance...")
            if not self.start_servo_application(test_path):
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

            # Run the actual test
            result = self.wait_for_test_completion_ohos(timeout_per_test, test_start_time)
            result["test_number"] = i
            result["test_path"] = test_path

            # Compare with expectations
            expectation_analysis = self.compare_with_expectations(test_path, result)
            result["expectation_analysis"] = expectation_analysis

            results.append(result)

            # Clean up session for next test
            self.cleanup()

            # Log with expectation info
            status_msg = f"Test {i} completed with status: {result['status']} and duration: {result['duration']:.1f}s"
            expectation_summary = expectation_analysis.get("summary", "UNKNOWN")
            if expectation_summary != "EXPECTED":
                status_msg += f" - Expectation: {expectation_summary}"
            logging.info(status_msg)

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
                self.webdriver_request("DELETE", f"/session/{self.session_id}", timeout=5)
            except Exception as e:
                logging.debug(f"Failed to delete WebDriver session {self.session_id} during cleanup: {e}")
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

        # Count results by status and expectation analysis
        status_counts = {}
        expectation_counts = {
            "EXPECTED": 0,
            "UNEXPECTED_CRASH": 0,
            "UNEXPECTED_TIMEOUT": 0,
            "UNEXPECTED_FAIL": 0,
            "UNEXPECTED_PASS": 0,
        }
        total_duration = 0
        total_pass = 0
        total_fail = 0

        for result in results:
            status = result.get("status", "UNKNOWN")
            status_counts[status] = status_counts.get(status, 0) + 1
            total_duration += result.get("duration", 0)
            total_pass += result.get("passCount", 0)
            total_fail += result.get("failCount", 0)

            # Track expectation analysis
            expectation_analysis = result.get("expectation_analysis", {})
            summary = expectation_analysis.get("summary", "UNKNOWN")

            if summary in expectation_counts:
                expectation_counts[summary] += 1
            else:
                # Unknown summary - classify as UNEXPECTED_FAIL as fallback
                expectation_counts["UNEXPECTED_FAIL"] += 1

        print(f"Total Tests: {len(results)}")
        print(f"Total Duration: {total_duration:.1f} seconds")
        print(f"Average Test Duration: {total_duration / len(results):.1f} seconds")

        print("\nStatus Breakdown:")
        for status, count in sorted(status_counts.items()):
            percentage = (count / len(results)) * 100
            print(f"  {status}: {count} ({percentage:.1f}%)")

        print("\nExpectation Analysis:")
        for exp_status, count in expectation_counts.items():
            if count > 0:
                percentage = (count / len(results)) * 100
                print(f"  {exp_status}: {count} ({percentage:.1f}%)")

        if total_pass > 0 or total_fail > 0:
            print("\nOverall Test Results:")
            print(f"  Total Assertions Passed: {total_pass}")
            print(f"  Total Assertions Failed: {total_fail}")
            if total_pass + total_fail > 0:
                pass_rate = (total_pass / (total_pass + total_fail)) * 100
                print(f"  Overall Pass Rate: {pass_rate:.1f}%")

        print(f"\n{'Test #':<6} {'Status':<18} {'Expected':<10} {'Duration':<10} {'Test Path'}")
        print("-" * 90)

        for result in results:
            test_num = result.get("test_number", "?")
            status = result.get("status", "UNKNOWN")
            duration = result.get("duration", 0)
            test_path = result.get("test_path", result.get("title", "Unknown")) or "Unknown"

            # Get expectation info
            expectation_analysis = result.get("expectation_analysis", {})
            expected_status = expectation_analysis.get("test_expected", "PASS")
            expectation_summary = expectation_analysis.get("summary", "UNKNOWN")

            # Mark unexpected results
            if expectation_summary not in ["EXPECTED"]:
                status += " (!)"

            # Truncate long paths
            if len(test_path) > 35:
                test_path = "..." + test_path[-32:]

            print(f"{test_num:<6} {status:<18} {expected_status:<10} {duration:<9.1f}s {test_path}")

        # Show detailed information for expectation mismatches
        unexpected_results = []
        for result in results:
            expectation_analysis = result.get("expectation_analysis", {})
            summary = expectation_analysis.get("summary", "UNKNOWN")
            if summary not in ["EXPECTED", "UNKNOWN"]:
                unexpected_results.append(result)

        if unexpected_results:
            print("\n" + "=" * 80)
            print(" EXPECTATION MISMATCHES")
            print("=" * 80)

            for result in unexpected_results:
                test_num = result.get("test_number", "?")
                test_path = result.get("test_path", result.get("title", "Unknown")) or "Unknown"
                expectation_analysis = result.get("expectation_analysis", {})

                print(f"\nTest #{test_num}: {test_path}")
                print(f"Expected: {expectation_analysis.get('test_expected', 'PASS')}")
                print(f"Actual: {expectation_analysis.get('test_actual', 'UNKNOWN')}")
                print(f"Summary: {expectation_analysis.get('summary', 'UNKNOWN')}")

                # Show subtest mismatches
                unexpected_subtests = expectation_analysis.get("unexpected_subtests", [])
                if unexpected_subtests:
                    print("  Unexpected Subtest Results:")
                    for subtest in unexpected_subtests[:5]:  # Limit to first 5
                        print(
                            f"    - {subtest.get('name', 'Unknown')}: expected {subtest.get('expected', '?')}, got {subtest.get('actual', '?')}"
                        )
                    if len(unexpected_subtests) > 5:
                        print(f"    ... and {len(unexpected_subtests) - 5} more")

        # Show detailed information for failed/problematic tests
        problematic_statuses = [
            "FAIL",
            "TIMEOUT",
            "CRASH",
            "ERROR",
            "RUNTIME_ERROR",
            "SESSION_ERROR",
            "WINDOW_ERROR",
            "NAVIGATION_ERROR",
            "COMPLETION_ERROR",  # Legacy status
        ]

        problematic_tests = [
            r
            for r in results
            if r.get("status") in problematic_statuses
            and r.get("expectation_analysis", {}).get("summary", "") == "EXPECTED"
        ]

        if problematic_tests:
            print("\n" + "=" * 80)
            print(" EXPECTED FAILURES (DETAILED)")
            print("=" * 80)

            for result in problematic_tests:
                test_num = result.get("test_number", "?")
                status = result.get("status", "UNKNOWN")
                test_path = result.get("test_path", result.get("title", "Unknown")) or "Unknown"

                print(f"\nTest #{test_num}: {test_path}")
                print(f"Status: {status} (as expected)")

                if result.get("failCount", 0) > 0:
                    print(f"Failed Assertions: {result.get('failCount', 0)}")

        print("\n" + "=" * 80)

    def print_unexpected_results_only(self, results: List[Dict[str, Any]]) -> None:
        """Print only tests with unexpected results (mismatching expectations)."""
        # Filter to only unexpected results
        unexpected_results = []
        for result in results:
            expectation_analysis = result.get("expectation_analysis", {})
            summary = expectation_analysis.get("summary", "UNKNOWN")
            if summary not in ["EXPECTED", "UNKNOWN"]:
                unexpected_results.append(result)

        if not unexpected_results:
            print("\n🎉 ALL TESTS MATCH EXPECTATIONS!")
            print("No unexpected test results found.")

            # Still show basic stats
            expected_count = sum(
                1 for r in results if r.get("expectation_analysis", {}).get("summary", "") == "EXPECTED"
            )

            print("\nSummary:")
            print(f"  Total tests: {len(results)}")
            print(f"  Expected results: {expected_count}")
            return

        print("\n" + "=" * 80)
        print(" UNEXPECTED TEST RESULTS")
        print("=" * 80)
        print(f"Found {len(unexpected_results)} tests with unexpected results out of {len(results)} total tests")

        # Group by expectation summary
        by_summary = {}
        for result in unexpected_results:
            summary = result.get("expectation_analysis", {}).get("summary", "UNKNOWN")
            if summary not in by_summary:
                by_summary[summary] = []
            by_summary[summary].append(result)

        for summary, tests in by_summary.items():
            print(f"\n{summary} ({len(tests)} tests):")
            print("-" * 60)

            for result in tests:
                test_path = result.get("test_path", "Unknown")
                expectation_analysis = result.get("expectation_analysis", {})
                expected = expectation_analysis.get("test_expected", "PASS")
                actual = expectation_analysis.get("test_actual", "UNKNOWN")

                print(f"  {test_path}")
                print(f"    Expected: {expected}, Got: {actual}")

                # Show subtest details if available
                unexpected_subtests = expectation_analysis.get("unexpected_subtests", [])
                if unexpected_subtests:
                    print(f"    Unexpected subtests: {len(unexpected_subtests)}")
                    for subtest in unexpected_subtests[:3]:  # Show first 3
                        subtest_name = subtest.get("name", "Unknown")
                        subtest_exp = subtest.get("expected", "?")
                        subtest_act = subtest.get("actual", "?")
                        print(f"      - {subtest_name}: expected {subtest_exp}, got {subtest_act}")
                    if len(unexpected_subtests) > 3:
                        print(f"      ... and {len(unexpected_subtests) - 3} more")
                print()

        print("=" * 80)


def main() -> int:
    parser = argparse.ArgumentParser(description="Run WPT tests on OHOS device")

    parser.add_argument("--test", required=True, help="Path to WPT test file or folder containing tests")
    parser.add_argument("--webdriver-port", type=int, default=7000, help="WebDriver server port")
    parser.add_argument("--wpt-server-port", type=int, default=8000, help="WPT server port")
    parser.add_argument("--timeout", type=int, default=15, help="Timeout per test in seconds")
    parser.add_argument("--verbose", action="store_true", help="Enable verbose logging")
    parser.add_argument(
        "--show-only-unexpected",
        action="store_true",
        help="Show only tests with unexpected results (mismatching expectations)",
    )

    args = parser.parse_args()

    log_level = logging.DEBUG if args.verbose else logging.INFO
    logging.basicConfig(level=log_level, format="%(asctime)s - %(levelname)s - %(message)s")

    controller = OHOSWebDriverController(args.webdriver_port, args.wpt_server_port)

    try:
        # Run pre-flight checks
        logging.info("Running pre-flight checks...")
        if not controller.check_device_connection():
            logging.error("Device connection check failed. Please ensure:")
            logging.error("1. OHOS device is connected via USB")
            logging.error("2. HDC is installed and in PATH")
            logging.error("3. Device is in developer mode")
            return 1

        # Determine tests to run - automatically detect if it's a file or folder
        test_paths = controller.discover_tests(args.test)
        if not test_paths:
            logging.error(f"No test files found at: {args.test}")
            return 1

        logging.info(f"Discovered {len(test_paths)} tests in: {args.test}")

        # Set up infrastructure once
        logging.info("Setting up test infrastructure...")

        if not controller.start_wpt_server():
            logging.error("Failed to start WPT server")
            return 1

        results = controller.run_tests(test_paths, args.timeout)

        # Print results using unified summary format
        if args.show_only_unexpected:
            controller.print_unexpected_results_only(results)
        else:
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
