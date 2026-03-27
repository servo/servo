#!/usr/bin/env python3

# Copyright 2026 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import os
import sys
import pathlib
import time
import common_function_for_servo_test
import subprocess
import random
from selenium import webdriver
from common_function_for_servo_test import MitmProxyRunType

WEBDRIVER_PORT = random.randrange(9000, 9999)
SERVO_BIN_PATH = "./target/release/servoshell"


def run_test(
    test_fn,
    test_name: str,
    use_mitmproxy: MitmProxyRunType = MitmProxyRunType.NOPROXY,
    session_history_max_length: int | None = None,
):
    if os.environ.get("CI") and use_mitmproxy == MitmProxyRunType.NOPROXY:
        # if we are in CI and nobody overrode our mitmproxy type we want to replay.
        print("Setting mitmproxy replay")
        use_mitmproxy = MitmProxyRunType.REPLAY

    dump_file = pathlib.Path("/tmp/mitmproxy-dump")
    if use_mitmproxy == MitmProxyRunType.REPLAY and not dump_file.is_file():
        print(f"Dump file {dump_file} did not exist. We will abort")
        return

    webdriver = start_servo(WEBDRIVER_PORT, SERVO_BIN_PATH, delay=2, session_history_max_length=session_history_max_length)
    if webdriver:
        webdriver.implicitly_wait(30)
        test_fn(webdriver)

    kill_servo()


def create_driver(timeout: int = 2) -> webdriver.Remote:
    return common_function_for_servo_test.create_driver(timeout=timeout, servo_url=f"http://127.0.0.1:{WEBDRIVER_PORT}")


def start_servo(webdriver_port: int, servo_path: str, delay: int = 0, session_history_max_length: int = 20) -> webdriver.Remote | None:
    """Start servo and create webdriver"""
    try:
        subprocess.Popen(
            [
                servo_path,
                f"--webdriver={webdriver_port}",
                f"--pref=session_history_max_length={session_history_max_length}"
            ]
        )
    except FileNotFoundError:
        print("The servo binary does not exist")
        return sys.exit(1)
    if delay > 0:
        time.sleep(delay)
    return create_driver(webdriver_port)


def kill_servo():
    subprocess.Popen(["killall", "servoshell"])
