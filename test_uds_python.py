#!/usr/bin/env python3
"""
Python integration tests for Unix Domain Socket server

Tests using the requests-unixsocket library to verify
Gunicorn is serving correctly over UDS.
"""

import json
import os
import signal
import subprocess
import sys
import time
import unittest
from pathlib import Path

# Try to import requests-unixsocket
try:
    import requests_unixsocket
except ImportError:
    print("Installing requests-unixsocket...")
    subprocess.check_call([sys.executable, "-m", "pip", "install", "-q", "requests-unixsocket"])
    import requests_unixsocket


class UnixSocketIntegrationTest(unittest.TestCase):
    """Integration tests for Gunicorn over Unix Domain Sockets"""

    SOCKET_DIR = "/tmp/servo-uds-pytest"
    SOCKET_PATH = f"{SOCKET_DIR}/test.sock"

    @classmethod
    def setUpClass(cls):
        """Start Gunicorn server before tests"""
        print("\n" + "=" * 60)
        print("Starting Gunicorn server on Unix socket...")
        print("=" * 60 + "\n")

        # Create socket directory
        Path(cls.SOCKET_DIR).mkdir(parents=True, exist_ok=True)

        # Remove old socket if it exists
        if os.path.exists(cls.SOCKET_PATH):
            os.remove(cls.SOCKET_PATH)

        # Start Gunicorn
        examples_dir = Path(__file__).parent / "examples"
        cls.gunicorn_process = subprocess.Popen(
            [
                "gunicorn",
                "--bind", f"unix:{cls.SOCKET_PATH}",
                "--workers", "2",
                "--log-level", "warning",
                "unix_socket_server:app"
            ],
            cwd=examples_dir,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )

        # Wait for socket to be created
        max_wait = 10
        for i in range(max_wait):
            if os.path.exists(cls.SOCKET_PATH):
                print(f"✓ Socket created: {cls.SOCKET_PATH}")
                break
            time.sleep(0.5)
        else:
            raise RuntimeError("Socket was not created in time")

        # Additional wait for server to be ready
        time.sleep(1)

        # Create requests session
        cls.session = requests_unixsocket.Session()

        print(f"✓ Gunicorn started (PID: {cls.gunicorn_process.pid})\n")

    @classmethod
    def tearDownClass(cls):
        """Stop Gunicorn server after tests"""
        print("\n" + "=" * 60)
        print("Stopping Gunicorn server...")
        print("=" * 60 + "\n")

        if cls.gunicorn_process:
            cls.gunicorn_process.terminate()
            try:
                cls.gunicorn_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                cls.gunicorn_process.kill()
                cls.gunicorn_process.wait()

        # Cleanup socket
        if os.path.exists(cls.SOCKET_PATH):
            os.remove(cls.SOCKET_PATH)

        print("✓ Cleanup complete\n")

    def _make_request(self, path="/", method="GET", **kwargs):
        """Helper to make Unix socket requests"""
        # Encode socket path for URL
        socket_url = f"http+unix://{self.SOCKET_PATH.replace('/', '%2F')}{path}"

        if method == "GET":
            return self.session.get(socket_url, **kwargs)
        elif method == "POST":
            return self.session.post(socket_url, **kwargs)
        else:
            raise ValueError(f"Unsupported method: {method}")

    def test_01_socket_exists(self):
        """Test that socket file exists"""
        self.assertTrue(os.path.exists(self.SOCKET_PATH))
        self.assertTrue(os.path.isdir(self.SOCKET_DIR) or os.path.issocket(self.SOCKET_PATH))

    def test_02_get_root_page(self):
        """Test GET request to root path"""
        response = self._make_request("/")
        self.assertEqual(response.status_code, 200)
        self.assertIn("Hello from Unix Socket Server", response.text)
        self.assertIn("text/html", response.headers.get("Content-Type", ""))

    def test_03_get_api_endpoint(self):
        """Test JSON API endpoint"""
        response = self._make_request("/api/data")
        self.assertEqual(response.status_code, 200)

        data = response.json()
        self.assertEqual(data["status"], "success")
        self.assertEqual(data["transport"], "unix_domain_socket")
        self.assertEqual(data["server"], "gunicorn + flask")
        self.assertIn("request", data)
        self.assertIn("headers", data)

    def test_04_get_test_page(self):
        """Test /test endpoint"""
        response = self._make_request("/test")
        self.assertEqual(response.status_code, 200)
        self.assertIn("Test Page", response.text)
        self.assertIn("Successfully loaded via Unix socket", response.text)

    def test_05_get_about_page(self):
        """Test /about endpoint"""
        response = self._make_request("/about")
        self.assertEqual(response.status_code, 200)
        self.assertIn("About This Demo", response.text)
        self.assertIn("Unix Domain Sockets", response.text)

    def test_06_404_error(self):
        """Test 404 error handling"""
        response = self._make_request("/nonexistent")
        self.assertEqual(response.status_code, 404)

    def test_07_post_request(self):
        """Test POST request to API"""
        payload = {"test": "data", "number": 42}
        response = self._make_request(
            "/api/data",
            method="POST",
            json=payload
        )
        self.assertEqual(response.status_code, 200)

        data = response.json()
        self.assertEqual(data["transport"], "unix_domain_socket")
        self.assertEqual(data["request"]["method"], "POST")

    def test_08_multiple_requests(self):
        """Test multiple concurrent requests"""
        results = []
        for i in range(10):
            response = self._make_request("/")
            results.append(response.status_code == 200)

        self.assertEqual(sum(results), 10, "All requests should succeed")

    def test_09_request_headers(self):
        """Test custom request headers"""
        headers = {"X-Custom-Header": "test-value"}
        response = self._make_request("/api/data", headers=headers)

        self.assertEqual(response.status_code, 200)
        data = response.json()

        # Check if our custom header was received
        self.assertIn("X-Custom-Header", data["headers"])
        self.assertEqual(data["headers"]["X-Custom-Header"], "test-value")

    def test_10_user_agent(self):
        """Test User-Agent header"""
        custom_ua = "Servo/1.0 (Unix Socket Test)"
        headers = {"User-Agent": custom_ua}
        response = self._make_request("/api/data", headers=headers)

        data = response.json()
        self.assertIn("User-Agent", data["headers"])
        self.assertEqual(data["headers"]["User-Agent"], custom_ua)

    def test_11_large_response(self):
        """Test handling of larger responses"""
        response = self._make_request("/")

        # Response should be reasonably sized HTML
        self.assertGreater(len(response.content), 500)
        self.assertLess(len(response.content), 50000)

    def test_12_content_encoding(self):
        """Test content encoding is properly set"""
        response = self._make_request("/")
        self.assertIn("Content-Type", response.headers)

    def test_13_api_structure(self):
        """Test API response has correct structure"""
        response = self._make_request("/api/data")
        data = response.json()

        required_keys = ["status", "message", "transport", "server", "request", "headers"]
        for key in required_keys:
            self.assertIn(key, data, f"API response missing key: {key}")

    def test_14_url_path_handling(self):
        """Test URL path is correctly processed"""
        response = self._make_request("/api/data")
        data = response.json()

        self.assertEqual(data["request"]["path"], "/api/data")

    def test_15_query_parameters(self):
        """Test query parameters are handled"""
        response = self._make_request("/api/data?foo=bar&test=123")
        data = response.json()

        self.assertEqual(data["request"]["args"]["foo"], "bar")
        self.assertEqual(data["request"]["args"]["test"], "123")


def run_tests():
    """Run the test suite"""
    # Create test suite
    loader = unittest.TestLoader()
    suite = loader.loadTestsFromTestCase(UnixSocketIntegrationTest)

    # Run with verbose output
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)

    # Print summary
    print("\n" + "=" * 60)
    print("TEST SUMMARY")
    print("=" * 60)
    print(f"Tests run: {result.testsRun}")
    print(f"Successes: {result.testsRun - len(result.failures) - len(result.errors)}")
    print(f"Failures: {len(result.failures)}")
    print(f"Errors: {len(result.errors)}")

    if result.wasSuccessful():
        print("\n✓ ALL TESTS PASSED")
        return 0
    else:
        print("\n✗ SOME TESTS FAILED")
        return 1


if __name__ == "__main__":
    sys.exit(run_tests())
