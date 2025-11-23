#!/usr/bin/env python3

# Copyright 2025 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from selenium.common import NoSuchWindowException, NoSuchElementException
from selenium.webdriver.support import expected_conditions
from selenium.webdriver.support.wait import WebDriverWait

import common_function_for_servo_test
from selenium.webdriver.common.by import By


def operator():
    driver = common_function_for_servo_test.create_driver()

    print("finding components ...")
    goal_css_selector1 = "#homeHero > div.hero-body > div.container > div > a:nth-child(1)"
    WebDriverWait(driver, 20, ignored_exceptions=[NoSuchWindowException, NoSuchElementException]).until(
        expected_conditions.presence_of_element_located((By.CSS_SELECTOR, goal_css_selector1))
    )
    goal_css_selector1 = "#homeHero > div.hero-body > div.container > h1"
    WebDriverWait(driver, 20, ignored_exceptions=[NoSuchWindowException, NoSuchElementException]).until(
        expected_conditions.presence_of_element_located((By.CSS_SELECTOR, goal_css_selector1))
    )
    print("find components successful!")


if __name__ == "__main__":
    common_function_for_servo_test.run_test(operator, "open_servo_org","https://servo.org")
