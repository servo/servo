#!/usr/bin/env python3

# Copyright 2025 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import argparse
import json
import os
from pathlib import PurePath
import subprocess
import time
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.common.options import ArgOptions
from urllib3.exceptions import ProtocolError
from selenium.common.exceptions import NoSuchElementException

from enum import Enum


class AbortReason(Enum):
    NotFound = 1
    Panic = 2


# the time in seconds we wait at most
MAX_WAIT_TIME = 60
# tests that currently do not return results
IGNORE_TESTS = [
    "tall-content-short-columns.html",
    "tall-content-short-columns-realistic.html",
]


def create_driver(timeout: int = 10) -> webdriver.Remote | None:
    """Create the webdriver connection."""
    print("Trying to create driver")
    options = ArgOptions()
    options.set_capability("browserName", "servo")
    driver = None
    start_time = time.time()
    while driver is None and time.time() - start_time < timeout:
        try:
            driver = webdriver.Remote(command_executor="http://127.0.0.1:7000", options=options)
        except (ConnectionError, ProtocolError):
            time.sleep(0.2)
        except Exception as e:
            print(f"Unexpected exception when creating webdriver: {e}, {type(e)}")
            time.sleep(1)
    print(
        "Established Webdriver connection",
    )
    return driver


def start_servo(servo_path: str) -> webdriver.Remote | None:
    """Start servo and create webdriver"""
    subprocess.Popen(
        [
            servo_path,
            "--webdriver",
        ]
    )
    return create_driver()


def kill_servo():
    subprocess.Popen(["killall", "servo"])


def test(s: str, driver: webdriver.Remote) -> tuple[str, str, str] | AbortReason:
    """Run a test by loading a website, and returning (avg, min, max).
    This will run for MAX_WAIT_TIME seconds and return as soon as the avg line exists in the log element"""

    print("Running: " + canonical_test_path(s, None))
    try:
        driver.get(s)
        for i in range(MAX_WAIT_TIME):
            element = driver.find_element(By.ID, "log")
            text = element.text
            result_lines = text.split("\n")
            # get the avg line or return None if it doesn't exist yet
            avg_line = next(filter(lambda x: "avg" in x, result_lines), None)
            min_line = next(filter(lambda x: "min" in x, result_lines), None)
            max_line = next(filter(lambda x: "max" in x, result_lines), None)
            if avg_line is not None and min_line is not None and max_line is not None:
                return (avg_line.split()[1], min_line.split()[1], max_line.split()[1])
            time.sleep(1)
    except NoSuchElementException:
        print("Could not find log?")
        return AbortReason.NotFound
    except Exception as e:
        print(f"Some other exception for this test case: {e}")
        return AbortReason.Panic
    return AbortReason.NotFound


def canonical_test_path(filePath: str, prepend: str | None) -> str:
    """Make the filepath into just the directory and name"""
    p = PurePath(filePath)
    parts = p.parts
    index = parts.index("perf_tests")
    if prepend:
        return prepend + "/" + "/".join(parts[index:])
    else:
        return "/".join(parts[index:])


def test_file(file_name: str) -> bool:
    return ".html" in file_name and (file_name not in IGNORE_TESTS)


def write_file(results):
    with open("results.json", "w") as f:
        json.dump(results, f)


def main():
    parser = argparse.ArgumentParser(description="Run Blink Perf Tests on Servo Instance.")
    parser.add_argument("servo_path", type=str, help="the servo binary")
    parser.add_argument(
        "-p",
        "--prepend",
        action="store",
        help="A value prepended to all results. Useful to distinguish between profiles.",
    )
    args = parser.parse_args()

    webdriver = start_servo(args.servo_path)
    final_result = {}
    time.sleep(2)
    if webdriver:
        webdriver.implicitly_wait(30)
        for root, dir, files in os.walk("../../tests/blink_perf_tests/perf_tests/layout"):
            for file in files:
                if test_file(file):
                    filePath = os.path.join(os.path.abspath(root), file)
                    result = test("file://" + filePath, webdriver)
                    if result == AbortReason.Panic:
                        print("Restarting servo")
                        start_servo(args.servo_path)
                    elif result == AbortReason.NotFound:
                        pass
                    else:
                        combined_result = {}
                        combined_result["value"] = result[0]
                        combined_result["lower_value"] = result[1]
                        combined_result["upper_value"] = result[2]

                        final_result[canonical_test_path(filePath, args.prepend)] = {"throughput": combined_result}

    print(final_result)
    write_file(final_result)
    kill_servo()


if __name__ == "__main__":
    main()
