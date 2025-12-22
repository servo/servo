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


import common_function_for_servo_test
import common_function_for_mossel


def operator():
    IMPLICIT_WAIT_TIME = 6
    driver = common_function_for_servo_test.create_driver()
    # This is used to wait for element retrieval if not found
    # and certain element click, element send key exceptions.
    driver.implicitly_wait(IMPLICIT_WAIT_TIME)
    common_function_for_mossel.load_mossel(driver)

    # Step 2. Click to close the pop-up
    common_function_for_mossel.close_popup(driver)

    time.sleep(2)

    # Step 3. Click to page: Categories
    common_function_for_mossel.click_category(driver)

    # Step 4. Find components
    common_function_for_mossel.identify_element_in_category(driver)


if __name__ == "__main__":
    common_function_for_servo_test.run_test(operator, "mossel_redirection")
