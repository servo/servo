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
from selenium.common.exceptions import NoSuchElementException
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from dataclasses import dataclass
import threading
from common_function_for_servo_test import create_driver, setup_hdc_forward, stop_servo, close_usb_popup

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
DIRECTORY = "tests/blink_perf_tests/perf_tests"

skipped_tests = [
    "large-table-with-collapsed-borders-and-no-colspans.html", # RAM
    "large-table-with-collapsed-borders-and-colspans-wider-than-table.html", # RAM
    "tall-content-short-columns-realistic.html", # JS Fatal
    "tall-content-short-columns.html", # JS Fatal
    "floats_10_1000.html" # timeout
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

        self.local_server = ThreadingHTTPServer(
            ("0.0.0.0", self.port),
            StaticHandler,
        )

        thread = threading.Thread(target=self.local_server.serve_forever, daemon=True)
        thread.start()

    def __exit__(self, *args):
        self.local_server.server_close()

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

def reset_tab_ram(driver: webdriver.Remote):
    original_tab = driver.current_window_handle
    driver.switch_to.new_window('tab')
    driver.get("about:blank")
    driver.switch_to.window(original_tab)
    driver.close()
    print(">>> windows has been resets")
    remaining_tab = driver.window_handles[0]
    driver.switch_to.window(remaining_tab)
    # time.sleep(.1)

def test(s: str, driver: webdriver.Remote, port: int, serve_path) -> TestResult | AbortReason:
    """Run a test by loading a website, and returning (avg, min, max).
    This will run for MAX_WAIT_TIME seconds and return as soon as the avg line exists in the log element"""

    # s = f"http://127.0.0.1:{port}/layout/{s}"
    s = f"http://127.0.0.1:{port}/{serve_path}/{s}"
    print(f">>> testing url: {s}")
    text = None
    try:
        before_get = time.perf_counter()
        # print(f">>> entered try at {before_get}")
        driver.get(s)
        after_get = time.perf_counter()
        # print(f">>> exited try at {after_get}, diff {after_get- before_get}")
        if after_get - before_get > 2:
            raise TimeoutError(f"Page loading took {(after_get - before_get):.2f}s (> 2s limit)")
        
        avg_line, min_line, max_line = None, None, None
        start_time, latest_time = time.perf_counter(), 0
        while (latest_time - start_time < 60):
            try:
                element = driver.find_element(By.ID, "log")
                latest_time = time.perf_counter()
                text = element.text
                # print(f">>> text:\n{text}\n<<<")
            except NoSuchElementException:
                time.sleep(1)
                continue
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
    with open("results.json", "w") as f:
        json.dump(results, f)

import csv
def run_tests(webdriver, port):
    skip_until = None
    # skip_until = "editing_append_single_line.html"

    final_result = {}
    with open("output.csv", "w", newline="", encoding="utf-8") as f:
        writer = csv.writer(f)
        writer.writerow(["name", "return"])

        for root, dir, files in os.walk("tests/blink_perf_tests/perf_tests/layout", onerror=oswalk_error):
            for file in files:
                dir[:] = [d for d in dir if d != "resources"]
                filePath = file
                if skip_until is None or skip_until in filePath:
                    skip_until = None
                    if filePath in skipped_tests:
                        continue
                    # reset_tab_ram(webdriver)
                    result = test(filePath, webdriver, port, get_serve_path_for_file(root, filePath))
                    print(f">>> result: {result}")
                    # get_memory_report(webdriver)
                    
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

def get_memory_report(driver: webdriver.Remote) -> str:
    original_tab = driver.current_window_handle
    driver.switch_to.new_window("tab")
    driver.get("about:memory")
    wait = WebDriverWait(driver, 200)
    measure_button = wait.until(
        EC.element_to_be_clickable((By.ID, "startButton"))
    )
    measure_button.click()
    reports = wait.until(
        lambda d: d.find_element(By.ID, "reports")
    )
    wait.until(
        lambda d: d.find_element(By.ID, "reports").text.strip() != ""
    )
    report_text =  reports.text
    driver.close()
    driver.switch_to.window(original_tab)
    return report_text

if __name__ == "__main__":
    hdc = HarmonyDeviceConnector()
    try:
        print("Stopping potential old servo instance ...")
        stop_servo()
        setup_hdc_forward(webdriver_port=WEBDRIVER_PORT, host_service_port=BLINK_PERF_FILES_SERVE_PORT)
        with LocalFileServe(BLINK_PERF_FILES_SERVE_PORT):
            print("Starting new servo instance...")
            cmd_str = f"aa start -a EntryAbility -b org.servo.servo -U {ABOUT_BLANK} --psn=--webdriver"
            hdc.cmd(cmd_str, timeout=10)
            with HarmonyDevicePerfMode(screen_timeout_seconds = 2 * 60 * 60):
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
