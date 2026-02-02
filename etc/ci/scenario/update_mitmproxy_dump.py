#!/usr/bin/env python3

# Copyright 2025 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import subprocess
import argparse
import time

import common_function_for_servo_test
import servo_speedometer
import servo_test_open_page_base
import servo_test_open_page_servo
import servo_test_redirection
import servo_test_slide


class colors:
    GREEN = "\033[92m"
    ENDC = "\033[0m"


def google_test():
    IMPLICIT_WAIT_TIME = 6
    PAGE_URL = "https://www.google.com"
    driver = common_function_for_servo_test.create_driver()
    driver.get(PAGE_URL)
    print("Page loaded.")
    driver.implicitly_wait(IMPLICIT_WAIT_TIME)


def run_test(name: str, f):
    print(colors.GREEN + name + colors.ENDC)
    try:
        common_function_for_servo_test.run_test(f, "servo", common_function_for_servo_test.MitmProxyRunType.RECORD)
    except Exception:
        print(f"Test {name} failed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Create a mitmdump file from google, servo and scenariotest")
    parser.add_argument("dump_file", type=str, help="the file we write the dump to")
    args = parser.parse_args()

    mitmproxy = subprocess.Popen(
        [
            "uv",
            "tool",
            "run",
            "--from",
            "mitmproxy",
            "mitmdump",
            "-w",
            args.dump_file,
            "-p",
            common_function_for_servo_test.MITMPROXY_PORT,
            # "--mode", "upstream:http://127.0.0.1:3128",
            "--set",
            "ssl_insecure=true",
        ]
    )
    print(f"Writing to {args.dump_file}")

    time.sleep(5)
    run_test("Running servo_test_open_page_servo", servo_test_open_page_servo.operator)
    run_test("Running servo_test_open_page_base", servo_test_open_page_base.operator)
    run_test("Running servo_test_redirection", servo_test_redirection.operator)
    run_test("Running servo_test_slide", servo_test_slide.operator)
    run_test("Running Servo Speedometer", servo_speedometer.run_speedometer)
    run_test("Google", google_test)

    print("FINISHED!")
    mitmproxy.terminate()
