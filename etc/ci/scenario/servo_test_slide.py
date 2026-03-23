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
from memory_usage_plotter import NonBlockingMemoryLogging, MemoryLoggingOptions


def operator():
    memory_logging_options = MemoryLoggingOptions(
        log_to_file=True, plot=True, pre_time=2, post_time=5, verbose=True, reset_tab=True
    )
    memory_logging = NonBlockingMemoryLogging(memory_logging_options)
    memory_logging.start()
    IMPLICIT_WAIT_TIME = 6
    driver = common_function_for_servo_test.create_driver()
    memory_logging.set_webdriver(driver)
    # This is used to wait for element retrieval if not found
    # and certain element click, element send key exceptions.
    driver.implicitly_wait(IMPLICIT_WAIT_TIME)
    memory_logging.event("load mossel")
    common_function_for_mossel.load_mossel(driver)

    # Step 2. Click to close the pop-up
    memory_logging.event("close popup")
    common_function_for_mossel.close_popup(driver)

    # Step 3. Click to page: Categories
    memory_logging.event("click category")
    common_function_for_mossel.click_category(driver)

    # Step 4. Find components which indicates the page has loaded, before sliding.
    common_function_for_mossel.identify_element_in_category(driver)

    print("Ready to swipe.")
    # Step 4. Slide and check if it actually happened.
    print("Screenshot before swiping.")
    before = driver.get_screenshot_as_base64()
    time.sleep(1)

    print("swiping...")
    memory_logging.event("swiping")
    cmd = ["hdc", "shell", "uinput -T -m 770 2000 770 930"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    time.sleep(5)

    print("Screenshot after swiping.")
    after = driver.get_screenshot_as_base64()

    if before == after:
        raise RuntimeError("The screenshots before and after sliding are the same; the slide failed.")
    memory_logging.stop()


if __name__ == "__main__":
    common_function_for_servo_test.run_test(operator, "mossel_slide", session_history_max_length=0)
