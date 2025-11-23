#!/usr/bin/env python3

# Copyright 2025 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import sys
import time
import subprocess
import os.path

from hdc_py.hdc import HarmonyDevicePerfMode
from selenium.common import NoSuchWindowException, NoSuchElementException
from selenium.webdriver.support import expected_conditions
from selenium.webdriver.support.wait import WebDriverWait

import common_function_for_servo_test
from selenium.webdriver.common.by import By


def operator():
    # 1. Open Servo
    print("Starting servo ...")
    cmd = ["hdc", "shell", "aa force-stop org.servo.servo"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    cmd = ["hdc", "shell", "aa start -a EntryAbility -b org.servo.servo --psn --webdriver"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    time.sleep(10)

    common_function_for_servo_test.setup_hdc_forward()
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
    try:
        with HarmonyDevicePerfMode():
            operator()
    except Exception as e:
        print(f"Scenario test {os.path.basename(__file__)} failed with error: {e} (exception: {type(e)})")
        sys.exit(1)

    common_function_for_servo_test.stop_servo()
