# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import sys, os
from selenium import webdriver

input = sys.argv[1]
output = sys.argv[2]

input = os.path.abspath(input)
output = os.path.abspath(output)

input = "file://" + input

print input
print output

driver = webdriver.Firefox()

driver.set_window_size(800, 1000)
driver.get(input)

screenshot = driver.get_screenshot_as_file(output)

driver.quit()
