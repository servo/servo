#!/usr/bin/env -S uv run --script

# Copyright 2026 The Servo Project Developers. See the COPYRIGHT
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
import argparse
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from hdc_py.hdc import HarmonyDeviceConnector, HarmonyDevicePerfMode
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.common.exceptions import NoSuchElementException
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from dataclasses import dataclass
import threading
import csv
from typing import Dict
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

# Tests we will skip because they are currently broken.
skipped_tests = [
    "large-table-with-collapsed-borders-and-no-colspans.html",  # RAM
    "large-table-with-collapsed-borders-and-colspans-wider-than-table.html",  # RAM
    "tall-content-short-columns-realistic.html",  # JS Fatal
    "tall-content-short-columns.html",  # JS Fatal
    "floats_10_1000.html",  # timeout
    "nested-percent-height-tables.html",
    "large-table-with-collapsed-borders-and-colspans.html",  # causes next test to fail
    "contain-content-style-change.html",
]

# `tests_that_hang_memory_report` do pass and contribute "ms" or "runs/s" to `results`
# but they also freeze the phone when trying to fetch memory report
# So, the tests in this list are not skipped in general, but the
# memory report for them is not included in the `results.json`
tests_that_hang_memory_report = [
    "large-grid.html",
    "layer-overhead.html",
    "css-contain-change-text.html",
    "fit-content-change-available-size-text.html",
    "css-contain-change-text-without-subtree-root.html",
]

### `tests_that_take_extra_time_to_load` tests take several seconds to load instead of default 0.05s - 2s
# can be overriden using `--page_loading_timeout` (in seconds)
tests_that_take_extra_time_to_load = [
    ("subtree-detaching.html", 20),
    ("abspos.html", 5),
    ("flexbox-row-stretch-height-definite.html", 18),
]

MAX_WAIT_TIME = 60

REGEX_RESULTS_LOG_ELEMENT = re.compile(
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


@dataclass(frozen=True)
class TestResult:
    value: float
    lower_value: float
    upper_value: float
    unit: str


class AbortReason(Enum):
    NotFound = 1
    Panic = 2


class LocalFileServe:
    def __init__(self, port: int, *args, **kwargs):
        self.local_server = None
        self.port = port

    def __enter__(self):
        print(f"Serving local test files on port {self.port}")

        class StaticHandler(SimpleHTTPRequestHandler):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, directory=DIRECTORY, **kwargs)

        self.local_server = ThreadingHTTPServer(
            ("127.0.0.1", self.port),
            StaticHandler,
        )

        thread = threading.Thread(target=self.local_server.serve_forever, daemon=True)
        thread.start()

    def __exit__(self, *args):
        if self.local_server is not None:
            self.local_server.server_close()


def get_serve_path_for_file(root: str, file: str) -> str:
    return root.split("tests/blink_perf_tests/perf_tests/")[1]


def run_single_test(
    test_file_name: str, driver: webdriver.Remote, port: int, serve_path, cli_args
) -> TestResult | AbortReason:
    """Run a test by loading a website, and returning (avg, min, max).
    This will run for MAX_WAIT_TIME seconds and return as soon as the avg line exists in the log element"""

    url = f"http://127.0.0.1:{port}/{serve_path}/{test_file_name}"
    print(f"Testing url: {url}")
    text = None
    try:
        before_get = time.perf_counter()
        driver.get(url)
        after_get = time.perf_counter()
        if cli_args.page_loading_timeout is None:
            special_case_time = next(
                (value for s, value in tests_that_take_extra_time_to_load if test_file_name in s), None
            )
            if special_case_time is not None:
                cli_args.page_loading_timeout = special_case_time
            else:
                cli_args.page_loading_timeout = 2
        if after_get - before_get > cli_args.page_loading_timeout:
            raise TimeoutError(
                f"Page loading took {(after_get - before_get):.2f}s (> {cli_args.page_loading_timeout}s limit)"
            )
        print(f">>> Page loading took {(after_get - before_get):.2f}s")

        avg_line, min_line, max_line = None, None, None
        start_time, latest_time = time.perf_counter(), 0
        while latest_time - start_time < 60:
            try:
                element = driver.find_element(By.ID, "log")
                latest_time = time.perf_counter()
                text = element.text
                # print(f">>> Text: \n{text}\n<<<\n")
            except NoSuchElementException:
                time.sleep(1)
                continue
            results_log_groups = REGEX_RESULTS_LOG_ELEMENT.search(text)
            if results_log_groups:
                avg_line = float(results_log_groups.group("avg"))
                min_line = float(results_log_groups.group("min"))
                max_line = float(results_log_groups.group("max"))
                unit = results_log_groups.group("unit")
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


def oswalk_error(error: OSError):
    print(error)
    sys.exit(1)


def write_file(results):
    with open("results.json", "w") as f:
        json.dump(results, f)


REGEX_MEMORY_REPORT = re.compile(
    r"""
    ^\s*(?P<explicit_value>\d+(?:\.\d+)?)\s+
    (?P<explicit_unit>MiB|GiB|KiB)\s+--\s+explicit\s*$

    (?:.*\n)*?

    ^\s*(?P<resident_value>\d+(?:\.\d+)?)\s+
    (?P<resident_unit>MiB|GiB|KiB)\s+--\s+resident\s*$

    (?:.*\n)*?

    ^\s*(?P<smaps_value>\d+(?:\.\d+)?)\s+
    (?P<smaps_unit>MiB|GiB|KiB)\s+--\s+resident-according-to-smaps\s*$
    """,
    re.MULTILINE | re.VERBOSE,
)


@dataclass(frozen=True)
class MemoryMetric:
    value: float
    unit: str


@dataclass(frozen=True)
class MemoryReport:
    explicit: MemoryMetric
    resident: MemoryMetric
    smaps: MemoryMetric


@dataclass(frozen=True)
class ParseError:
    message: str


def parse_memory_report(
    text: str,
) -> MemoryReport | ParseError:
    match = REGEX_MEMORY_REPORT.search(text)
    if not match:
        return ParseError("Could not parse memory report (explicit/resident/smaps)")

    try:
        explicit = MemoryMetric(
            value=float(match.group("explicit_value")),
            unit=match.group("explicit_unit"),
        )
        resident = MemoryMetric(
            value=float(match.group("resident_value")),
            unit=match.group("resident_unit"),
        )
        smaps = MemoryMetric(
            value=float(match.group("smaps_value")),
            unit=match.group("smaps_unit"),
        )
    except (ValueError, KeyError) as exc:
        return ParseError(f"Invalid numeric value: {exc}")

    return MemoryReport(
        explicit=explicit,
        resident=resident,
        smaps=smaps,
    )


_UNIT_TO_MIB = {
    "KiB": 1.0 / 1024.0,
    "MiB": 1.0,
    "GiB": 1024.0,
}


def _to_mib(value: float, unit: str) -> float:
    if unit not in _UNIT_TO_MIB:
        raise ValueError(f"Unsupported unit: {unit}")
    return value * _UNIT_TO_MIB[unit]


def memory_report_to_bencher_metrics(
    report: MemoryReport,
) -> Dict[str, Dict[str, float]]:
    """
    Convert MemoryReport into Bencher-compatible metric dict.

    Output shape:
    {
        "memory_explicit_mib": {"value": float},
        "memory_resident_mib": {"value": float},
        "memory_resident_smaps_mib": {"value": float},
    }
    """

    explicit_mib = _to_mib(
        report.explicit.value,
        report.explicit.unit,
    )
    resident_mib = _to_mib(
        report.resident.value,
        report.resident.unit,
    )
    smaps_mib = _to_mib(
        report.smaps.value,
        report.smaps.unit,
    )

    return {
        "memory_explicit_mib": {
            "value": explicit_mib,
        },
        "memory_resident_mib": {
            "value": resident_mib,
        },
        "memory_resident_smaps_mib": {
            "value": smaps_mib,
        },
    }


def verbose_print(str_to_print: str, is_verbose: bool = False):
    if is_verbose:
        print(str_to_print)


def run_tests(port, cli_args):
    skip_until = cli_args.skip_until

    final_result = {}
    csv_file, csv_writer = None, None
    if cli_args and cli_args.extra_csv:
        csv_file = open("blink_perf_test_logs.csv", "w", newline="", encoding="utf-8")
        csv_writer = csv.writer(csv_file)
        csv_writer.writerow(["name", "return"])

    try:
        for root, dir, files in os.walk("tests/blink_perf_tests/perf_tests/layout", onerror=oswalk_error):
            for file in files:
                dir[:] = [d for d in dir if d != "resources"]
                filePath = file
                if skip_until is None or skip_until in filePath:
                    if not cli_args.single_test:
                        skip_until = None
                    if filePath in skipped_tests:
                        continue
                    print("Starting new servo instance...")
                    cmd_str = f"aa start -a EntryAbility -b org.servo.servo -U {ABOUT_BLANK} --psn=--webdriver --psn=--pref=session_history_max_length=1"
                    hdc.cmd(cmd_str, timeout=10)
                    with HarmonyDevicePerfMode(screen_timeout_seconds=2 * 60 * 60):
                        close_usb_popup(hdc)
                        webdriver = create_driver(timeout=1)
                        if webdriver is None:
                            continue

                        try:
                            result = run_single_test(
                                filePath, webdriver, port, get_serve_path_for_file(root, filePath), cli_args=cli_args
                            )

                            verbose_print(f"result: {result}", cli_args.verbose)

                            if result == AbortReason.NotFound or result == AbortReason.Panic:
                                if csv_writer:
                                    csv_writer.writerow([filePath, "error"])
                            else:
                                combined_result = {
                                    "value": result.value,
                                    "lower_value": result.lower_value,
                                    "upper_value": result.upper_value,
                                }
                                parent_dir = get_serve_path_for_file(root, filePath)

                                bencher_unit = None
                                if result.unit == "ms":
                                    bencher_unit = "ms"
                                elif result.unit == "runs/s":
                                    bencher_unit = "throughput"
                                else:
                                    bencher_unit = "other"

                                if cli_args.memory_report and filePath not in tests_that_hang_memory_report:
                                    memory_report = get_memory_report_str(webdriver)

                                    if isinstance(memory_report, ParseError):
                                        print(f"Test memory parsing failed: {memory_report.message}")
                                    else:
                                        verbose_print(
                                            f"memory after test {memory_report}",
                                            cli_args.verbose,
                                        )
                                        metrics = memory_report_to_bencher_metrics(memory_report)
                                    final_result[f"perf_tests/{parent_dir}/{filePath}"] = {
                                        bencher_unit: combined_result,
                                        **metrics,
                                    }
                                else:
                                    final_result[f"perf_tests/{parent_dir}/{filePath}"] = {
                                        bencher_unit: combined_result
                                    }
                                if csv_writer:
                                    csv_writer.writerow([filePath, result.value])
                        finally:
                            webdriver.quit()
                        if csv_writer:
                            csv_file.flush()
                    stop_servo()
        print(f"final_result {final_result}")
    finally:
        if csv_file:
            csv_file.close()

    write_file(final_result)


def get_memory_report_str(driver: webdriver.Remote) -> MemoryReport | ParseError:
    report_text = None
    try:
        driver.get("about:memory")
        wait = WebDriverWait(driver, 5)
        measure_button = wait.until(EC.element_to_be_clickable((By.ID, "startButton")))
        measure_button.click()
        reports = wait.until(lambda d: d.find_element(By.ID, "reports"))
        wait.until(lambda d: d.find_element(By.ID, "reports").text.strip() != "")
        report_text = reports.text
    except Exception as e:
        return ParseError(message=str(e))

    return parse_memory_report(report_text)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-m",
        "--memory_report",
        action="store_true",
        help="Include memory report in results.json",
    )
    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="Add more prints",
    )
    parser.add_argument(
        "-e", "--extra_csv", action="store_true", help="Creates output.csv that contains skipper and failed tests"
    )
    parser.add_argument(
        "-s",
        "--single_test",
        action="store_true",
        help="Executes only first test. Can be used with --skip-until to run only one specific test",
    )
    parser.add_argument(
        "--skip_until",
        type=str,
        default=None,
        help="Skips all test until specified one, and if --single_test is not set, continues until the end.",
    )
    parser.add_argument(
        "--page_loading_timeout",
        type=int,
        default=None,
        help="Pages should load very fast, but if takes longer (than default 2s or specified), the result parsing is skipped",
    )
    args = parser.parse_args()
    hdc = HarmonyDeviceConnector()
    try:
        print("Stopping potential old servo instance ...")
        stop_servo()
        setup_hdc_forward(webdriver_port=WEBDRIVER_PORT, host_service_port=BLINK_PERF_FILES_SERVE_PORT)
        with LocalFileServe(BLINK_PERF_FILES_SERVE_PORT):
            run_tests(BLINK_PERF_FILES_SERVE_PORT, cli_args=args)
    except Exception as e:
        print(f"Test failed with error: {e} (exception: {type(e)})")
        hdc.screenshot("Blink_perf_test_error.jpg")
        stop_servo()
        sys.exit(1)
    print("\033[32mTest Succeeded.\033[0m")
    stop_servo()
