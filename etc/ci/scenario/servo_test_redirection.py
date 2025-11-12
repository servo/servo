#!/usr/bin/env python3

# Copyright 2018 The Servo Project Developers. See the COPYRIGHT
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
import common_function_for_servo_test
from selenium.webdriver.common.by import By


def operator():
    # Step 1. Open mossel
    cmd = ["hdc", "shell", "aa force-stop org.servo.servo"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    cmd = ["hdc", "shell", "aa start -a EntryAbility -b org.servo.servo -U https://m.huaweimossel.com --psn --webdriver"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    time.sleep(10)

    driver = common_function_for_servo_test.setup_hdc_forward()
    if driver is not False:
        # Step 2. Click to close the pop-up
        birthday_ = driver.find_element(By.CSS_SELECTOR,
                                        "#app > uni-app > uni-page > uni-page-wrapper > uni-page-body > uni-view > uni-view:nth-child(5) > uni-view.m-popup.m-popup_transition.m-mask_show.m-mask_fade.m-popup_push.m-fixed_mid > uni-view > uni-view > uni-button:nth-child(1)")
        birthday_.click()
        time.sleep(1)

        # Step 3. Click 'Categories'
        cmd = ["hdc", "shell", "uinput -T -c 380 2556"]
        subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        time.sleep(2)

        # Step 4. Check component
        try:
            driver.find_element(By.CSS_SELECTOR,
                                     "#app > uni-app > uni-page > uni-page-wrapper > uni-page-body > uni-view > uni-view.sort-main.m-flex.m-bgWhite > uni-scroll-view > div > div > div > uni-view.item.active")
            cmd = ["hdc", "shell", "aa force-stop org.servo.servo"]
            subprocess.run(cmd, capture_output=True, text=True, timeout=10)
            return True
        except:
            cmd = ["hdc", "shell", "aa force-stop org.servo.servo"]
            subprocess.run(cmd, capture_output=True, text=True, timeout=10)
            return False
    else:
        return False

if __name__ == '__main__':
    result = operator()
    print(result)
    if not result:
        sys.exit(1)
