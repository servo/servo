import os
import sys
import random
import unittest

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test

repo_root = os.path.abspath(os.path.join(__file__, "../../.."))
sys.path.insert(1, os.path.join(repo_root, "tools", "webdriver"))
from webdriver import exceptions

class WindowSizeTest(base_test.WebDriverBaseTest):

    def test_set_and_get_window_size(self):
        self.driver.get(self.webserver.where_is("command_contexts/res/first-page.html"))

        initial_dimensions = self.driver.get_window_size()

        new_dimensions = {
            "height": initial_dimensions["height"] - 100,
            "width": initial_dimensions["width"] - 100}

        try:
            self.driver.set_window_size(new_dimensions["height"], new_dimensions["width"])

            actual_dimensions = self.driver.get_window_size()

            self.assertDictEqual(new_dimensions, actual_dimensions)
        except exceptions.UnsupportedOperationException:
            pass


if __name__ == "__main__":
    unittest.main()
