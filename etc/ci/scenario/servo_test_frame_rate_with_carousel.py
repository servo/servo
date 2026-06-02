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

from selenium.common import NoSuchWindowException, NoSuchElementException
from selenium.webdriver.support import expected_conditions
from selenium.webdriver.support.wait import WebDriverWait

import common_function_for_servo_test
from selenium.webdriver.common.by import By


def operator():
    driver = common_function_for_servo_test.create_driver()

    popup_css_selector = (
        "#app > uni-app > uni-page > uni-page-wrapper > uni-page-body > uni-view "
        "> uni-view:nth-child(5) "
        "> uni-view.m-popup.m-popup_transition.m-mask_show.m-mask_fade.m-popup_push.m-fixed_mid "
        "> uni-view > uni-view > uni-button:nth-child(1)"
    )
    print("Waiting for popup to appear ...")
    WebDriverWait(driver, 20, ignored_exceptions=[NoSuchWindowException, NoSuchElementException]).until(
        expected_conditions.presence_of_element_located((By.CSS_SELECTOR, popup_css_selector))
    )

    # Step 2. Click to close the pop-up
    popup = driver.find_element(By.CSS_SELECTOR, popup_css_selector)
    popup.click()
    print("Closed the popup.")

    driver.implicitly_wait(10)
    time.sleep(5)

    # Step 3. Click 'Categories'
    print("Click 'Categories'.")
    cmd = ["hdc", "shell", "uinput -T -c 380 2556"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    time.sleep(5)

    print("Waiting for page to finish loading...")
    WebDriverWait(driver, 10).until(lambda driver: driver.execute_script("return document.readyState") == "complete")
    print("document.readyState == complete")

    # Step 4. Click on the first product
    print("Searching for 'first_good'...")
    first_good = driver.find_element(By.CSS_SELECTOR, "#goodsGroup > uni-view:nth-child(1) > img")
    print("Clicking 'first_good'.")
    first_good.click()

    time.sleep(5)

    # Step 5. Click for details
    print("Searching for 'first_good_details'...")
    content = driver.find_element(
        By.CSS_SELECTOR,
        "#app > uni-app > uni-page > uni-page-wrapper > uni-page-body > uni-view > uni-view.m-tab.product.m-flex__level > uni-text:nth-child(2) > span",
    )
    print("Clicking 'first_good_details'.")
    content.click()
    time.sleep(5)

    # Step 6. Trace
    print("Starting hitrace!")
    cmd = [
        "hdc",
        "shell",
        "hitrace -b 81920 -t 10 ace app ohos ability graphic -o /data/local/tmp/my_trace.html",
    ]
    process = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    time.sleep(5)

    print("Swiping...")
    cmd = ["hdc", "shell", "uinput -T -m 770 2000 770 770 30"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=2)

    process.wait(timeout=15)
    stdout, stderr = process.communicate()
    print("hitrace command complete. Output:")
    print(stdout.decode())
    if stderr:
        print("Error message:")
        print(stderr.decode())

    frame_rate = common_function_for_servo_test.calculate_frame_rate()
    print(f"framerate is {frame_rate}")

    if frame_rate < 115:
        raise RuntimeError(f"Actual frame rate is {frame_rate}, expected >= 115")


if __name__ == "__main__":
    common_function_for_servo_test.run_test(operator, "mossel_frame_rate_with_carousel", "https://m.huaweimossel.com")
