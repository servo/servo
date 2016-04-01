# -*- mode: python; fill-column: 100; comment-column: 100; -*-

import os
import sys
import unittest
import time

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test
from selenium.common import exceptions
from selenium.webdriver.common.keys import Keys
from selenium.webdriver.common.action_chains import ActionChains


class tabbingTest(base_test.WebDriverBaseTest):
    def test_open_close_tab(self):
        self.driver.get(self.webserver.where_is("windows/res/win1.html"))
        self.driver.find_element_by_tag_name("div").click()
        h = self.driver.window_handles
        self.assertEquals(2, len(h))
        self.driver.switch_to.window(h[1])
        try:
            self.driver.switch_to.window("does not exist")
            self.fail("NoSuchWindowException expected")
        except exceptions.NoSuchWindowException:
            pass
        self.driver.close()

if __name__ == "__main__":
    unittest.main()
