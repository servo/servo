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
from typing import Dict, Optional, Any


class OHOSWebDriverController:
    """Controller for running WebDriver tests on OHOS devices using HTTP API."""

    def __init__(self, webdriver_port: int = 7000, wpt_server_port: int = 8000) -> None:
        self.webdriver_port = webdriver_port
        self.wpt_server_port = wpt_server_port
        self.session_id: Optional[str] = None
        self.wpt_server_process: Optional[subprocess.Popen] = None

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

    def run_test(self, test_path: str) -> Dict[str, Any]:
        """Run a single WPT test."""
        try:
            if not self.create_session():
                return {
                    "status": "ERROR",
                    "title": "",
                    "details": "Failed to create WebDriver session",
                    "passCount": 0,
                    "failCount": 0,
                    "failingTests": [],
                }

            if not self.create_window():
                return {
                    "status": "ERROR",
                    "title": "",
                    "details": "Failed to create window",
                    "passCount": 0,
                    "failCount": 0,
                    "failingTests": [],
                }

            test_url = f"http://localhost:{self.wpt_server_port}/{test_path}"

            logging.info(f"Navigating URL: {test_url}")

            navigation_result = self.navigate_to_url(test_url, timeout=5)

            if navigation_result:
                logging.info("Navigation completed, proceeding to test completion check")
            else:
                logging.warning("Navigation may have failed, but continuing with test completion check")

            return self.wait_for_test_completion_ohos()

        except Exception as e:
            logging.error(f"Error running test: {e}")
            return {
                "status": "ERROR",
                "title": "",
                "details": str(e),
                "passCount": 0,
                "failCount": 0,
                "failingTests": [],
            }

    def wait_for_test_completion_ohos(self, timeout: int = 30) -> Dict[str, Any]:
        """OHOS test completion handling"""
        try:
            logging.info("OHOS test completion handling...")

            logging.info("Waiting for page to load and test to complete...")
            for i in range(6):
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
                                "details": result.get("bodyText", "")[:200] + "..."
                                if len(result.get("bodyText", "")) > 200
                                else result.get("bodyText", ""),
                                "passCount": result.get("passCount", 0),
                                "failCount": result.get("failCount", 0),
                                "failingTests": result.get("failingTests", []),
                            }
                        else:
                            logging.info(
                                f"Test still running, status: {result.get('status')}, body preview: {result.get('bodyText', '')[:100]}..."
                            )
                except Exception as api_error:
                    logging.debug(f"API request failed: {api_error}")

            # If we get here, either test timed out or API is completely unresponsive
            logging.warning("WebDriver API appears to be unresponsive - this is a known OHOS limitation")

            # Take screenshot for debugging
            screenshot_path = f"test_output/servo_ohos_screenshot_{int(time.time())}.jpeg"
            self.take_screenshot(screenshot_path)

            return {
                "status": "INDETERMINATE",
                "title": "OHOS WebDriver Limitation",
                "details": (
                    "Test was successfully loaded on OHOS device, but WebDriver API became "
                    "unresponsive. Please check the test result manually on the device screen,"
                    "or refer to the screenshot at of Desktop at: " + screenshot_path
                ),
                "passCount": 0,
                "failCount": 0,
                "failingTests": [],
            }

        except Exception as e:
            logging.error(f"Error in OHOS test completion handling: {e}")

            # Take screenshot for debugging on error
            screenshot_path = f"test_output/servo_ohos_error_screenshot_{int(time.time())}.jpeg"
            self.take_screenshot(screenshot_path)

            return {
                "status": "ERROR",
                "title": "",
                "details": str(e),
                "passCount": 0,
                "failCount": 0,
                "failingTests": [],
            }

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


def main() -> int:
    parser = argparse.ArgumentParser(description="Run a single WPT test on OHOS device")
    parser.add_argument("--test", required=True, help="Path to WPT test (relative to tests/wpt/tests/)")
    parser.add_argument("--webdriver-port", type=int, default=7000, help="WebDriver server port")
    parser.add_argument("--wpt-server-port", type=int, default=8000, help="WPT server port")
    parser.add_argument("--verbose", action="store_true", help="Enable verbose logging")

    args = parser.parse_args()

    log_level = logging.DEBUG if args.verbose else logging.INFO
    logging.basicConfig(level=log_level, format="%(asctime)s - %(levelname)s - %(message)s")

    controller = OHOSWebDriverController(args.webdriver_port, args.wpt_server_port)

    try:
        logging.info("Killing any existing servo instances and starting fresh...")

        try:
            subprocess.run(["hdc", "shell", "killall org.servo.servo"], capture_output=True, text=True, timeout=10)
            logging.info("Killed existing servo processes")
        except Exception as e:
            logging.debug(f"killall command failed (may be expected): {e}")

        try:
            subprocess.run(
                ["hdc", "shell", "aa force-stop org.servo.servo"], capture_output=True, text=True, timeout=10
            )
            logging.info("Force stopped servo application")
        except Exception as e:
            logging.debug(f"force-stop command failed (may be expected): {e}")

        try:
            subprocess.run(
                ["hdc", "shell", "aa start -a EntryAbility -b org.servo.servo"],
                capture_output=True,
                text=True,
                timeout=15,
            )
            logging.info("Started servo application")
            time.sleep(3)
        except Exception as e:
            logging.error(f"Failed to start servo application: {e}")
            return 1

        logging.info("Setting up test infrastructure...")

        if not controller.setup_hdc_forwarding():
            logging.error("Failed to set up HDC forwarding")
            return 1

        controller.setup_wpt_server_access()

        if not controller.start_wpt_server():
            logging.error("Failed to start WPT server")
            return 1

        logging.info(f"Running test: {args.test}")
        result = controller.run_test(args.test)

        print("\nTest Results:")
        print("=" * 50)
        print(f"Status: {result['status']}")
        print(f"Title: {result['title']}")

        if "passCount" in result and "failCount" in result:
            total_tests = result["passCount"] + result["failCount"]
            print(f"Total Tests: {total_tests}")
            print(f"Passed: {result['passCount']}")
            print(f"Failed: {result['failCount']}")

            if result["failCount"] > 0 and "failingTests" in result and result["failingTests"]:
                print(f"\nFailing Tests ({len(result['failingTests'])} extracted):")
                print("-" * 50)
                actual_count = 0
                for i, failing_test in enumerate(result["failingTests"], 1):
                    if isinstance(failing_test, dict):
                        test_name = failing_test.get("name", "Unknown")
                        error_msg = failing_test.get("error", "No error message")
                    else:
                        test_name = str(failing_test)
                        error_msg = "No error message"

                    actual_count += 1
                    print(f"{actual_count}. Test: {test_name}")
                    print(f"   Error: {error_msg}")
                    print()

                    if actual_count >= result["failCount"]:
                        break

        return 0 if result["status"] == "PASS" else 1

    except KeyboardInterrupt:
        logging.info("Test interrupted by user")
        return 1
    except Exception as e:
        logging.error(f"Unexpected error: {e}")
        return 1
    finally:
        controller.cleanup()

        logging.info("Cleaning up servo instances...")
        try:
            subprocess.run(["hdc", "shell", "killall org.servo.servo"], capture_output=True, text=True, timeout=10)
            logging.info("Killed servo processes")
        except Exception as e:
            logging.debug(f"killall command failed during cleanup: {e}")

        try:
            subprocess.run(
                ["hdc", "shell", "aa force-stop org.servo.servo"], capture_output=True, text=True, timeout=10
            )
            logging.info("Force stopped servo application")
        except Exception as e:
            logging.debug(f"force-stop command failed during cleanup: {e}")

    # This should never be reached
    return 1


if __name__ == "__main__":
    sys.exit(main())
