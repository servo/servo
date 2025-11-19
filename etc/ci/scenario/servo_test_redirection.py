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
from selenium.common import NoSuchWindowException, NoSuchElementException
from selenium.webdriver.support import expected_conditions
from selenium.webdriver.support.wait import WebDriverWait

import common_function_for_servo_test
from selenium.webdriver.common.by import By


def operator():
    # Step 1. Open mossel
    print("Starting mossel ...")
    cmd = ["hdc", "shell", "aa force-stop org.servo.servo"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    cmd = [
        "hdc",
        "shell",
        "aa start -a EntryAbility -b org.servo.servo -U https://m.huaweimossel.com --psn --webdriver",
    ]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)

    common_function_for_servo_test.setup_hdc_forward()
    driver = common_function_for_servo_test.create_driver()

    # Step 2. Click to close the pop-up
    popup_css_selector = "#app > uni-app > uni-page > uni-page-wrapper > uni-page-body > uni-view > uni-view:nth-child(5) > uni-view.m-popup.m-popup_transition.m-mask_show.m-mask_fade.m-popup_push.m-fixed_mid > uni-view > uni-view > uni-button:nth-child(1)"
    print("Waiting for popup to appear ...")
    WebDriverWait(driver, 20, ignored_exceptions=[NoSuchWindowException, NoSuchElementException]).until(
        expected_conditions.presence_of_element_located((By.CSS_SELECTOR, popup_css_selector))
    )
    time.sleep(1)

    birthday_ = driver.find_element(By.CSS_SELECTOR, popup_css_selector)
    birthday_.click()
    print("Closed the popup")

    # Step 3. Click to page: Categories
    print("Clicking 'Categories' element")
    cmd = ["hdc", "shell", "uinput -T -c 380 2556"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    time.sleep(2)

    print("Finding components ...")
    goal_css_selector = "#app > uni-app > uni-page > uni-page-wrapper > uni-page-body > uni-view > uni-view.sort-main.m-flex.m-bgWhite > uni-scroll-view > div > div > div > uni-view.item.active"
    WebDriverWait(driver, 20, ignored_exceptions=[NoSuchWindowException, NoSuchElementException]).until(
        expected_conditions.presence_of_element_located((By.CSS_SELECTOR, goal_css_selector))
    )
    print("Find components successful!")


if __name__ == "__main__":
    try:
        operator()
    except Exception as e:
        print(f"Scenario test {os.path.basename(__file__)} failed with error: {e} (exception: {type(e)})")
        sys.exit(1)

    common_function_for_servo_test.stop_servo()
