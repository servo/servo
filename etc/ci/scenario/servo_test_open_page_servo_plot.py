#!/usr/bin/env python3

# Copyright 2025 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from selenium.common import NoSuchElementException

import common_function_for_servo_test
from selenium.webdriver.common.by import By
from memory_usage_plotter import NonBlockingMemoryLogging
from selenium import webdriver

###
# This script is a modified `etc/ci/scenario/servo_test_open_page_servo.py`
# It asks for extra `operator` arguments to be able to additionally support
# adding custom events to the plot and to record memory after page reset
###


###
# the command
# `PLOT=1 UV_PROJECT=etc/ci/scenario uv run etc/ci/scenario/servo_test_open_page_servo.py`
# can also produce the plot, but would lack custom events and reconding after reset
###
def operator(driver: webdriver, memory_logging: NonBlockingMemoryLogging):
    IMPLICIT_WAIT_TIME = 6
    PAGE_URL = "https://servo.org"
    memory_logging.event("Get servo.org")
    driver.get(PAGE_URL)

    print("Page loaded.")
    memory_logging.event("Page loaded.")
    # This is used to wait for element retrieval if not found
    # and certain element click, element send key exceptions.
    driver.implicitly_wait(IMPLICIT_WAIT_TIME)

    print("Finding components ...")
    goal_css_selector1 = "#homeHero > div.hero-body > div.container > div > a:nth-child(1)"

    goal_css_selector2 = "#homeHero > div.hero-body > div.container > h1"

    try:
        driver.find_element(By.CSS_SELECTOR, goal_css_selector1)
        driver.find_element(By.CSS_SELECTOR, goal_css_selector2)
    except NoSuchElementException:
        raise NoSuchElementException("Components not found. Test failed.")

    print("Find components successful!")


if __name__ == "__main__":
    common_function_for_servo_test.run_test(operator, "open_servo_org", use_memory_logging=True)
