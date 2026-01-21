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
            # "--mode", "upstream:http://127.0.0.1:3128",
            "--set",
            "ssl_insecure=true",
        ]
    )
    print(f"Writing to {args.dump_file}")

    print(colors.GREEN + "Running servo_test_open_page_servo" + colors.ENDC)
    common_function_for_servo_test.run_test(
        servo_test_open_page_servo.operator, "servo", common_function_for_servo_test.MitmProxyRunType.RECORD
    )

    print(colors.GREEN + "Running servo_test_open_page_base" + colors.ENDC)
    common_function_for_servo_test.run_test(
        servo_test_open_page_base.operator, "base", common_function_for_servo_test.MitmProxyRunType.RECORD
    )

    print(colors.GREEN + "Running servo_test_redirection" + colors.ENDC)
    common_function_for_servo_test.run_test(
        servo_test_redirection.operator, "mossel_redirection", common_function_for_servo_test.MitmProxyRunType.RECORD
    )

    print(colors.GREEN + "Running servo_test_slide" + colors.ENDC)
    common_function_for_servo_test.run_test(
        servo_test_slide.operator, "mossel_slide", common_function_for_servo_test.MitmProxyRunType.RECORD
    )

    print(colors.GREEN + "Servo Speedometer" + colors.ENDC)
    common_function_for_servo_test.run_test(
        servo_speedometer.run_speedometer, "speedometer", common_function_for_servo_test.MitmProxyRunType.RECORD
    )

    print(colors.GREEN + "Google" + colors.ENDC)
    common_function_for_servo_test.run_test(
        google_test, "google", common_function_for_servo_test.MitmProxyRunType.RECORD
    )

    print("FINISHED!")
    mitmproxy.terminate()
