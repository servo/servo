#!/usr/bin/env python3

# Copyright 2025 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import os
from datetime import datetime

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
    date = datetime.today().strftime("%Y-%m-%d")
    if os.environ["http_proxy"] is not None:
        proxy = os.environ["http_proxy"]
        print(
            f"Start mitmproxy by hand like this: \n \
              uv run mitmproxy --mode upstream:{proxy} -w dump-{date} --set ssl_insecure=true"
        )
    else:
        print(
            f"Start mitmproxy by hand like this: \n \
              uv run mitmproxy --mode upstream:http://127.0.0.1:3128 -w dump-{date} --set ssl_insecure=true"
        )
    input("Press Enter to continue...")

    print(colors.GREEN + "Running servo_test_open_page_servo" + colors.ENDC)
    common_function_for_servo_test.run_test(servo_test_open_page_servo.operator, "servo")

    print(colors.GREEN + "Running servo_test_open_page_base" + colors.ENDC)
    common_function_for_servo_test.run_test(servo_test_open_page_base.operator, "base")

    print(colors.GREEN + "Running servo_test_redirection" + colors.ENDC)
    common_function_for_servo_test.run_test(servo_test_redirection.operator, "mossel_redirection")

    print(colors.GREEN + "Running servo_test_slide" + colors.ENDC)
    common_function_for_servo_test.run_test(servo_test_slide.operator, "mossel_slide")

    print(colors.GREEN + "Servo Speedometer" + colors.ENDC)
    common_function_for_servo_test.run_test(servo_speedometer.run_speedometer, "speedometer")

    print(colors.GREEN + "Google" + colors.ENDC)
    common_function_for_servo_test.run_test(google_test, "google")

    print("FINISHED!")
    print("Go back to mitmproxy window and press q")
