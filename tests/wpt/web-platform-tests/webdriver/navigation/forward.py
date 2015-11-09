import unittest
import sys
import os

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test


class ForwardTest(base_test.WebDriverBaseTest):
    # Get a static page that must be the same upon refresh
    def test_forward(self):
        self.driver.get(self.webserver.where_is('navigation/res/forwardStart.html'))
        self.driver.get(self.webserver.where_is('navigation/res/forwardNext.html'))
        nextbody = self.driver.find_element_by_css_selector("body").text
        self.driver.back()
        currbody = self.driver.find_element_by_css_selector("body").text
        self.assertNotEqual(nextbody, currbody)
        self.driver.forward()
        currbody = self.driver.find_element_by_css_selector("body").text
        self.assertEqual(nextbody, currbody)


if __name__ == '__main__':
    unittest.main()
