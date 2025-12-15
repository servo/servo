#!/usr/bin/env python3

# Copyright 2025 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import time
import subprocess


import common_function_for_servo_test
import common_function_for_mossel

from selenium.common import NoSuchElementException
from selenium.webdriver.common.by import By


def operator():
    IMPLICIT_WAIT_TIME = 30
    PAGE_URL = "https://m.huaweimossel.com"
    driver = common_function_for_servo_test.create_driver()
    driver.get(PAGE_URL)

    print("Page loaded.")
    # This is used to wait for element retrieval if not found
    # and certain element click, element send key exceptions.
    driver.implicitly_wait(IMPLICIT_WAIT_TIME)

    # Step 2. Click to close the pop-up
    common_function_for_mossel.close_popup(driver)

    # Step 3. Click to page: Categories
    print("Clicking 'Categories' element.")
    # TODO: Replace with Element click to be robust against screen size changes.
    cmd = ["hdc", "shell", "uinput -T -c 380 2556"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)

    # Step 4. Find components which indicates the page has loaded, before sliding.
    print("Finding components ...")
    target_css_selector = (
        "#app > uni-app > uni-page > uni-page-wrapper > uni-page-body > uni-view "
        "> uni-view.sort-main.m-flex.m-bgWhite > uni-scroll-view > div > div > div "
        "> uni-view.item.active"
    )

    try:
        driver.find_element(By.CSS_SELECTOR, target_css_selector)
    except NoSuchElementException:
        raise NoSuchElementException("Components not found. Test failed.")

    print("Find components successful! Ready to swipe.")
    # Step 4. Slide and check if it actually happened.
    print("Screenshot before swiping.")
    before = driver.get_screenshot_as_base64()
    time.sleep(1)

    print("swiping...")
    cmd = ["hdc", "shell", "uinput -T -m 770 2000 770 930"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    time.sleep(5)

    print("Screenshot after swiping.")
    after = driver.get_screenshot_as_base64()

    if before == after:
        raise RuntimeError("The screenshots before and after sliding are the same; the slide failed.")


if __name__ == "__main__":
    common_function_for_servo_test.run_test(operator, "mossel_slide")
