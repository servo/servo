# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import annotations
from concurrent.futures import Future
import http.server
import logging
import os
import os.path
import socket
import socketserver
import subprocess
import sys
import time
from threading import Thread
from typing import Optional
import unittest

from geckordp.actors.events import Events
from geckordp.actors.resources import Resources
from geckordp.actors.web_console import WebConsoleActor

from .console import ConsoleTests
from .debugger import DebuggerTests
from .inspector import InspectorTests
from .network import NetworkTests
from .utils import LOG_REQUESTS, Devtools


TEST_MODULES = [ConsoleTests, DebuggerTests, InspectorTests, NetworkTests]


class DevtoolsTestHarness:
    # /path/to/servo/python/servo
    script_path = None
    servo_binary: Optional[str] = None
    servoshell = None
    base_urls = None
    web_servers = None
    web_server_threads = None

    # Sets `base_url` and `web_server` and `web_server_thread`.
    @classmethod
    def setUpClass(cls):
        assert cls.base_urls is None and cls.web_servers is None and cls.web_server_threads is None
        test_dir = os.path.join(DevtoolsTestHarness.script_path, "devtools_tests")
        num_servers = 2
        base_urls = [Future() for i in range(num_servers)]
        cls.web_servers = [None for i in range(num_servers)]
        cls.web_server_threads = [None for i in range(num_servers)]

        class Handler(http.server.SimpleHTTPRequestHandler):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, directory=test_dir, **kwargs)

            def log_message(self, format, *args):
                if LOG_REQUESTS:
                    return super().log_message(format, *args)

        def server_thread(index):
            # There may be client sockets still open in TIME_WAIT state from previous tests, and they may stay open for
            # some minutes. Set SO_REUSEADDR to avoid bind failure with EADDRINUSE in these cases.
            # <https://stackoverflow.com/questions/14388706>
            socketserver.TCPServer.allow_reuse_address = True
            # Listen on all IPv4 interfaces, port 10000 + index.
            web_server = socketserver.TCPServer(("127.0.0.1", 10000 + index), Handler)
            base_url = f"http://127.0.0.1:{web_server.server_address[1]}"
            base_urls[index].set_result(base_url)
            cls.web_servers[index] = web_server
            web_server.serve_forever()

        # Start a web server for the test.
        for index in range(num_servers):
            thread = Thread(target=server_thread, args=[index])
            cls.web_server_threads[index] = thread
            thread.start()
        cls.base_urls = [base_url.result(1) for base_url in base_urls]

    # Sets `servoshell`.
    def run_servoshell(self, *, url):
        # Change this setting if you want to debug Servo.
        os.environ["RUST_LOG"] = "error,devtools=warn"

        # Run servoshell.
        self.servoshell = subprocess.Popen(
            [f"{DevtoolsTestHarness.servo_binary}", "--headless", "--devtools=6080", url]
        )

        sleep_per_try = 1 / 8  # seconds
        remaining_tries = 5 / sleep_per_try  # 5 seconds
        while True:
            print(".", end="", file=sys.stderr)
            stream = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            try:
                stream.connect(("127.0.0.1", 6080))
                stream.recv(4096)  # FIXME: without this, geckordp RDPClient.connect() may fail
                stream.shutdown(socket.SHUT_RDWR)
                print("+", end="", file=sys.stderr)
                break
            except Exception:
                time.sleep(sleep_per_try)
                assert remaining_tries > 0
                remaining_tries -= 1
                continue

    def tearDown(self):
        # Terminate servoshell, but do not stop the web servers.
        if self.servoshell is not None:
            self.servoshell.terminate()
            try:
                self.servoshell.wait(timeout=3)
            except subprocess.TimeoutExpired:
                print("Warning: servoshell did not terminate", file=sys.stderr)
                self.servoshell.kill()
            self.servoshell = None

    @classmethod
    def tearDownClass(cls):
        # Stop the web servers.
        if cls.web_servers is not None:
            for web_server in cls.web_servers:
                web_server.shutdown()
                web_server.server_close()
            cls.web_servers = None
        if cls.web_server_threads is not None:
            for web_server_thread in cls.web_server_threads:
                web_server_thread.join()
            cls.web_server_threads = None
        if cls.base_urls is not None:
            cls.base_urls = None

    def get_test_path(self, path: str) -> str:
        return os.path.join(DevtoolsTestHarness.script_path, os.path.join("devtools_tests", path))

    def evaluate_and_capture_console_log_output(self, js: str, timeout: float = 1) -> dict:
        with Devtools.connect() as devtools:
            devtools.watcher.watch_resources([Resources.CONSOLE_MESSAGE])

            console = WebConsoleActor(devtools.client, devtools.targets[0]["consoleActor"])
            evaluation_result = Future()

            async def on_resource_available(data):
                for resource in data["array"]:
                    if resource[0] != "console-message":
                        continue
                    evaluation_result.set_result(resource[1][0])
                    return

            devtools.client.add_event_listener(
                devtools.targets[0]["actor"], Events.Watcher.RESOURCES_AVAILABLE_ARRAY, on_resource_available
            )

            console.evaluate_js_async(js)
            return evaluation_result.result(timeout)


def run_tests(script_path, servo_binary: str, test_names: list[str]):
    DevtoolsTestHarness.script_path = script_path
    DevtoolsTestHarness.servo_binary = servo_binary

    harness = DevtoolsTestHarness()
    for module in TEST_MODULES:
        module.harness = harness

    verbosity = 1 if logging.getLogger().level >= logging.WARN else 2
    loader = unittest.TestLoader()
    if test_names:
        patterns = []
        # unittest.main() `-k` treats any `pattern` not containing `*` like `*pattern*`
        for pattern in test_names:
            if "*" in pattern:
                patterns.append(pattern)
            else:
                patterns.append(f"*{pattern}*")
        loader.testNamePatterns = patterns

    suite = unittest.TestSuite()
    for module in TEST_MODULES:
        suite.addTests(loader.loadTestsFromTestCase(module))

    print(f"Running {suite.countTestCases()} tests:", file=sys.stderr)
    for test in suite:
        print(f"- {test}", file=sys.stderr)
    print(file=sys.stderr)

    DevtoolsTestHarness.setUpClass()
    try:
        return unittest.TextTestRunner(verbosity=verbosity).run(suite).wasSuccessful()
    finally:
        DevtoolsTestHarness.tearDownClass()
