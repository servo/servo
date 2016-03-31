import os
import sys
import unittest

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test
from selenium.common import exceptions


class PageLoadTimeoutTest(base_test.WebDriverBaseTest):
    def test_should_timeout_on_page_load_taking_too_long(self):
        self.driver.set_page_load_timeout(0.01)
        with self.assertRaises(exceptions.TimeoutException):
            self.load_page()

    def test_should_not_timeout_on_page_load(self):
        self.driver.set_page_load_timeout(30)
        self.load_page()
        pass

    def load_page(self):
        self.driver.get(self.webserver.where_is('timeouts/res/page_load_timeouts_tests.html'))


if __name__ == "__main__":
    unittest.main()
