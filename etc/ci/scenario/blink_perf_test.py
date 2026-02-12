#!/usr/bin/env -S uv run --script

# Copyright 2025 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import random
import subprocess
from enum import Enum
import os
import time
import sys
import re
import json
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from hdc_py.hdc import HarmonyDeviceConnector, HarmonyDevicePerfMode
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.common.options import ArgOptions
from selenium.common.exceptions import NoSuchElementException
from urllib3.exceptions import ProtocolError
from dataclasses import dataclass
import threading


class PortMapResult(Enum):
    SUCCESSFUL = (1,)
    PORT_EXISTS = (2,)
    FORWARD_FAILED = (3,)

    def is_success(self) -> bool:
        return self == PortMapResult.SUCCESSFUL or self == PortMapResult.PORT_EXISTS


WEBDRIVER_PORT = 7000
BLINK_PERF_FILES_SERVE_PORT = random.randrange(7150, 9000)
SERVO_URL = f"http://127.0.0.1:{WEBDRIVER_PORT}"
ABOUT_BLANK = "about:blank"
DIRECTORY = "../../../tests/blink_perf_tests/perf_tests"

skipped_tests = [
    "large-table-with-collapsed-borders-and-no-colspans.html",
    "large-table-with-collapsed-borders-and-colspans-wider-than-table.html"
]

class LocalFileServe:
    def __init__(self, port: int, *args, **kwargs):
        self.local_server = None
        self.port = port

    def __enter__(self):
        print(f">>> Serving local test files on port {self.port}")

        class StaticHandler(SimpleHTTPRequestHandler):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, directory=DIRECTORY, **kwargs)

        # handler = partial(SimpleHTTPRequestHandler, directory = DIRECTORY)
        self.local_server = ThreadingHTTPServer(
            ("0.0.0.0", self.port),
            StaticHandler,
        )

        # self.local_server.serve_forever()
        thread = threading.Thread(target=self.local_server.serve_forever, daemon=True)
        thread.start()

    def __exit__(self, *args):
        print(f">>> *args: {args}")
        print(f">>> Closing local test file server on port {self.port}")
        self.local_server.server_close()


def create_driver(timeout: int = 10) -> webdriver.Remote:
    print("Trying to create driver")
    options = ArgOptions()
    options.set_capability("browserName", "servo")
    driver = None
    start_time = time.time()
    while driver is None and time.time() - start_time < timeout:
        try:
            driver = webdriver.Remote(command_executor=SERVO_URL, options=options)
        except (ConnectionError, ProtocolError):
            time.sleep(0.2)
        except Exception as e:
            print(f"Unexpected exception when creating webdriver: {e}, {type(e)}")
            time.sleep(1)
    print(
        f"Established Webdriver connection in {time.time() - start_time}s",
    )
    return driver


def port_forward(port: int | str, reverse: bool) -> PortMapResult:
    cmd = ["hdc", "fport", "ls"]
    output = subprocess.check_output(cmd, encoding="utf-8")
    if f"tcp:{port}" in output:
        return PortMapResult.PORT_EXISTS

    cmd = []
    if reverse:
        cmd = ["hdc", "rport", f"tcp:{port}", f"tcp:{port}"]
    else:
        cmd = ["hdc", "fport", f"tcp:{port}", f"tcp:{port}"]
    print(f"Setting up HDC port forwarding: {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    if result.stdout.startswith("[Fail]TCP Port listen failed"):
        print("Forward failed")
        return PortMapResult.FORWARD_FAILED
    elif result.stdout.startswith("[Fail]"):
        print("Forward failed other way")
        raise RuntimeError(f"HDC port forwarding failed with: {result.stdout}")

    print("Port forward successful")
    return PortMapResult.SUCCESSFUL


def setup_hdc_forward(timeout: int = 5):
    """
    set hdc forward
    :return: If successful, return driver; If failed, return False
    """
    for v in ("HTTP_PROXY", "http_proxy", "HTTPS_PROXY", "https_proxy"):
        os.environ.pop(v, None)

    start_time = time.time()
    while time.time() - start_time < timeout:
        try:
            if (
                port_forward(WEBDRIVER_PORT, False).is_success()
                and port_forward(BLINK_PERF_FILES_SERVE_PORT, True).is_success()
            ):
                return
            time.sleep(0.2)
        except FileNotFoundError:
            print("HDC command not found. Make sure OHOS SDK is installed and hdc is in PATH.")
            raise
        except subprocess.TimeoutExpired:
            print(f"HDC port forwarding timed out on port {WEBDRIVER_PORT}")
            raise
        except Exception as e:
            print(f"failed to setup HDC forwarding: {e}")
            raise
    raise TimeoutError("HDC port forwarding timed out")


def stop_servo():
    """stop servo application"""
    print("Prepare to stop Test Application...")
    cmd = ["hdc", "shell", "aa force-stop org.servo.servo"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    print("Stop Test Application successful!")


def close_usb_popup(hdc: HarmonyDeviceConnector):
    """
    When connecting an OpenHarmony device, a system pop-up will be opened on the device,
    asking the user to confirm which USB mode should be used for the connection.
    This pop-up overlays servo, and will hence disturb some inputs, and affect screenshots.
    """
    try:
        if not is_servo_window_focused(hdc):
            print("The focused window does not belong to servo. Sending back-event to try and close the window.")
            # The USB pop-up can be dismissed by simulating the back key-event.
            hdc.cmd("uitest uiInput keyEvent Back")
        if not is_servo_window_focused(hdc):
            print("The focused window still isn't servo. Giving up.")
    except Exception as e:
        print(f"Internal error trying to close the USB pop-up overlay: {e}. Ignoring...")


def is_servo_window_focused(hdc: HarmonyDeviceConnector) -> bool:
    completed_process = hdc.cmd("hidumper -s WindowManagerService -a '-a'", capture_output=True, encoding="utf-8")
    output = str(completed_process.stdout)
    lines = output.splitlines()
    focused_window = None
    servo_window_id = None
    for line in lines:
        if line.lower().startswith("focus window:"):
            focused_window = int(line.split(":")[1].strip())
        if line.lower().startswith("servo"):
            # The table format is as follows:
            # WindowName DisplayId Pid WinId
            servo_window_id = int(line.split()[3])
        if line.lower().startswith("windowname"):
            table_headers = line.split()
            assert table_headers[0].lower() == "windowname"
            assert table_headers[3].lower() == "winid"
        if focused_window is not None and servo_window_id is not None:
            break
    if focused_window is None or servo_window_id is None:
        raise RuntimeError("Could not find focused window or servo window id.")
    return focused_window == servo_window_id


@dataclass(frozen=True)
class TestResult:
    value: float
    lower_value: float
    upper_value: float
    unit: str


class AbortReason(Enum):
    NotFound = 1
    Panic = 2


MAX_WAIT_TIME = 60

_PATTERN = re.compile(
    r"""
Time:\s+
values\s+[0-9.,\s]+(?P<unit>ms|runs\/s)\s+
avg\s+(?P<avg>[0-9.]+)\s+(?P=unit)\s+
median\s+[0-9.]+\s+(?P=unit)\s+
stdev\s+[0-9.]+\s+(?P=unit)\s+
min\s+(?P<min>[0-9.]+)\s+(?P=unit)\s+
max\s+(?P<max>[0-9.]+)\s+(?P=unit)
""",
    re.VERBOSE,
)


def get_serve_path_for_file(root: str, file: str) -> str:
    return root.split("tests/blink_perf_tests/perf_tests/")[1]


def test(s: str, driver: webdriver.Remote, port: int, serve_path) -> TestResult | AbortReason:
    """Run a test by loading a website, and returning (avg, min, max).
    This will run for MAX_WAIT_TIME seconds and return as soon as the avg line exists in the log element"""

    # s = f"http://127.0.0.1:{port}/layout/{s}"
    s = f"http://127.0.0.1:{port}/{serve_path}/{s}"
    print(f">>> testing url: {s}")
    text = None
    try:
        driver.get(s)
        avg_line, min_line, max_line = None, None, None
        for i in range(MAX_WAIT_TIME):
            element = driver.find_element(By.ID, "log")
            text = element.text
            m = _PATTERN.search(text)
            if m:
                avg_line = float(m.group("avg"))
                min_line = float(m.group("min"))
                max_line = float(m.group("max"))
                unit = m.group("unit")
            if avg_line is not None and min_line is not None and max_line is not None and unit is not None:
                return TestResult(value=avg_line, lower_value=min_line, upper_value=max_line, unit=unit)
            time.sleep(1)
    except NoSuchElementException:
        print("Could not find log?")
        return AbortReason.NotFound
    except Exception as e:
        print(f"Some other exception for this test case: {e}")
        return AbortReason.Panic
    return AbortReason.NotFound


PREPEND = "PHONE"


def oswalk_error(error: OSError):
    print(error)
    sys.exit(1)


def write_file(results):
    with open("../../../results.json", "w") as f:
        json.dump(results, f)

import csv
def run_tests(webdriver, port):
    skip_until = "nested-percent-height-tables.html"

    final_result = {}
    with open("../../../output.csv", "w", newline="", encoding="utf-8") as f:
        writer = csv.writer(f)
        writer.writerow(["name", "return"])

        for root, dir, files in os.walk("../../../tests/blink_perf_tests/perf_tests/layout", onerror=oswalk_error):
            for file in files:
                filePath = file
                if skip_until is None or skip_until in filePath:
                    skip_until = None
                    if filePath in skipped_tests:
                        continue
                    # print(f">>> ROOT: {root}\n>>> dir: {dir}\n>>> files: {files}\n<<<")
                    result = test(filePath, webdriver, port, get_serve_path_for_file(root, filePath))
                    print(f">>> result: {result}")
                    if result == AbortReason.NotFound or result == AbortReason.Panic:
                        writer.writerow([filePath, "error"])
                        pass
                    else:
                        combined_result = {}
                        combined_result["value"] = result.value
                        combined_result["lower_value"] = result.lower_value
                        combined_result["upper_value"] = result.upper_value
                        parent_dir = get_serve_path_for_file(root, filePath)
                        if result.unit == "ms":
                            final_result[f"perf_tests/{parent_dir}/{filePath}"] = {"ms": combined_result}
                        else:
                            final_result[f"perf_tests/{parent_dir}/{filePath}"] = {"throughput": combined_result}
                        writer.writerow([filePath, result.value])
                    f.flush()
        print(f">>> final_result {final_result}")
    write_file(final_result)


if __name__ == "__main__":
    hdc = HarmonyDeviceConnector()
    try:
        print("Stopping potential old servo instance ...")
        stop_servo()
        setup_hdc_forward()
        with LocalFileServe(BLINK_PERF_FILES_SERVE_PORT):
            print("Starting new servo instance...")
            cmd_str = f"aa start -a EntryAbility -b org.servo.servo -U {ABOUT_BLANK} --psn=--webdriver"
            hdc.cmd(cmd_str, timeout=10)
            with HarmonyDevicePerfMode(screen_timeout_seconds = 1 * 60 * 60):
                close_usb_popup(hdc)
                print(">>> Creating webdriver")
                wd = create_driver()
                print(">>> Running the tests")
                run_tests(wd, BLINK_PERF_FILES_SERVE_PORT)
    except Exception as e:
        print(f"Test failed with error: {e} (exception: {type(e)})")
        hdc.screenshot("Blink_perf_test_error.jpg")
        stop_servo()
        sys.exit(1)
    print("\033[32mTest Succeeded.\033[0m")
    stop_servo()
