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


def operator():
    IMPLICIT_WAIT_TIME = 10
    PAGE_URL = "https://servo.org"
    driver = common_function_for_servo_test.create_driver()
    driver.get(PAGE_URL)

    print("Page loaded.")
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
    common_function_for_servo_test.run_test(operator, "open_servo_org")
