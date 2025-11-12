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
import common_function_for_servo_test
from selenium.webdriver.common.by import By


def operator():
    # 1. Open Servo
    cmd = ["hdc", "shell", "aa force-stop org.servo.servo"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    cmd = ["hdc", "shell", "aa start -a EntryAbility -b org.servo.servo --psn --webdriver"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    time.sleep(10)

    driver = common_function_for_servo_test.setup_hdc_forward()
    # Step 2. Check component
    if driver is not False:
        try:
            driver.find_element(By.CSS_SELECTOR, "#homeHero > div.hero-body > div.container > div > a:nth-child(1)") and driver.find_element(By.CSS_SELECTOR, "#homeHero > div.hero-body > div.container > h1")
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
