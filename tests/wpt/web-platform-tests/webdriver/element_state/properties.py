import os
import sys
import unittest

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test


class GetElementPropertiesTest(base_test.WebDriverBaseTest):
    def test_get_element_text(self):
        self.driver.get(self.webserver.where_is("element_state/res/elements_text.html"))
        e = self.driver.find_element_by_name("name")
        self.assertEquals("name", e.text)


if __name__ == "__main__":
    unittest.main()
