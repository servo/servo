# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import http.server
import os
import socket
import socketserver
import subprocess
import sys
import time
from concurrent.futures import Future
from subprocess import TimeoutExpired
from threading import Thread

import pytest

from . import utils

WAIT_BETWEEN_ATTEMPTS = 1 / 8  # seconds
CONNECTION_TIMEOUT = 5  # seconds


def pytest_addoption(parser):
    parser.addoption("--servo-binary", help="Path to the servoshell binary")
    parser.addoption("--script-path", help="Path to the servo python library")


@pytest.fixture(scope="session")
def servo_binary(request):
    binary = request.config.getoption("--servo-binary")
    if not binary:
        pytest.fail("The --servo-binary option must be specified")
    return binary


@pytest.fixture(scope="session")
def script_path(request):
    path = request.config.getoption("--script-path")
    if not path:
        pytest.fail("The --script-path option must be specified")
    return path


@pytest.fixture(scope="session")
def web_server_urls(script_path):
    test_dir = os.path.join(script_path, "devtools_tests")
    base_urls = [Future() for _ in range(len(utils.WEB_SERVERS))]
    web_servers = [None for _ in range(len(utils.WEB_SERVERS))]
    web_server_threads = [None for _ in range(len(utils.WEB_SERVERS))]

    class Handler(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, directory=test_dir, **kwargs)

        def log_message(self, format, *args):
            if utils.LOG_REQUESTS:
                return super().log_message(format, *args)

    def server_thread(i):
        # There may be client sockets still open in TIME_WAIT state from previous tests, and they may stay open for
        # some minutes. Set SO_REUSEADDR to avoid bind failure with EADDRINUSE in these cases.
        # <https://stackoverflow.com/questions/14388706>
        socketserver.TCPServer.allow_reuse_address = True

        # Listen on all IPv4 interfaces.
        port = utils.WEB_SERVERS[i]
        web_server = socketserver.TCPServer((utils.SERVER_ADDRESS, port), Handler)

        base_urls[i].set_result(f"http://{utils.SERVER_ADDRESS}:{port}")
        web_servers[i] = web_server
        web_server.serve_forever()

    # Start a web server for the test.
    for i in range(len(utils.WEB_SERVERS)):
        thread = Thread(target=server_thread, args=[i])
        web_server_threads[i] = thread
        thread.start()

    # Return the urls for the servers.
    yield [url.result(1) for url in base_urls]

    # Stop the servers and the threads.
    for server in web_servers:
        if server:
            server.shutdown()
            server.server_close()
    for thread in web_server_threads:
        if thread:
            thread.join()


@pytest.fixture
def run_servoshell(servo_binary):
    process = None

    def run(*, url):
        nonlocal process

        # Change this setting if you want to debug Servo.
        os.environ["RUST_LOG"] = "error,devtools=warn"

        # Run servoshell.
        process = subprocess.Popen([servo_binary, "--headless", f"--devtools={utils.DEVTOOLS_PORT}", url])

        # Try to connect to the devtools server.
        for attempt in range(int(CONNECTION_TIMEOUT / WAIT_BETWEEN_ATTEMPTS)):
            print(".", end="", file=sys.stderr)
            try:
                with socket.create_connection((utils.SERVER_ADDRESS, utils.DEVTOOLS_PORT)) as stream:
                    stream.recv(4096)  # FIXME: geckordp workaround
                    stream.shutdown(socket.SHUT_RDWR)
                print("+", end="", file=sys.stderr, flush=True)
                return process
            except Exception:
                time.sleep(WAIT_BETWEEN_ATTEMPTS)
        raise TimeoutError(
            f"Couldn't connect to the devtools server at {utils.SERVER_ADDRESS}:{utils.DEVTOOLS_PORT} in {CONNECTION_TIMEOUT}s"
        )

    yield run

    # Terminate servoshell.
    if process:
        process.terminate()
        try:
            process.wait(timeout=CONNECTION_TIMEOUT)
        except TimeoutExpired:
            print("Warning: servoshell did not terminate", file=sys.stderr)
            process.kill()
