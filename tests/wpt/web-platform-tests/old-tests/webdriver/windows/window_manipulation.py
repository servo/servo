# -*- mode: python; fill-column: 100; comment-column: 100; -*-

import os
import sys
import unittest

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test
from selenium.common import exceptions


class WindowingTest(base_test.WebDriverBaseTest):
    def test_maximize(self):
        #self.driver.get(self.webserver.where_is("windows/res/win1.html"))
        self.driver.maximize_window()

    def test_window_size_manipulation(self):
        #self.driver.get(self.webserver.where_is("windows/res/win1.html"))
        self.driver.set_window_size(400, 400)
        window_size = self.driver.get_window_size()
        self.assertTrue("width" in window_size)
        self.assertTrue("height" in window_size)
        self.assertEquals({"width": 400, "height":400}, window_size)

        """
        todo: make that work
        see: https://w3c.github.io/webdriver/webdriver-spec.html#setwindowsize
                result = self.driver.set_window_size(100, 100)
                self.assertTrue("status" in result)
                self.assertEquals(result["status"], 500)
        """

    def test_window_position_manipulation(self):
        #self.driver.get(self.webserver.where_is("windows/res/win1.html"))
        self.driver.set_window_position(400, 400)
        window_position = self.driver.get_window_position()
        self.assertTrue("x" in window_position)
        self.assertTrue("y" in window_position)
        self.assertEquals({"x": 400, "y": 400}, window_position)


if __name__ == "__main__":
    unittest.main()
