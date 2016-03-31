import os
import sys
import unittest

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test
from selenium.common import exceptions


class ImplicitWaitsTests(base_test.WebDriverBaseTest):
    def setUp(self):
        self.driver.get(self.webserver.where_is('timeouts/res/implicit_waits_tests.html'))

    def test_find_element_by_id(self):
        add = self.driver.find_element_by_css_selector("#adder")
        self.driver.implicitly_wait(3)
        add.click()
        self.driver.find_element_by_css_selector("#box0")  # All is well if this doesn't throw.

    def test_should_still_fail_to_find_an_element_when_implicit_waits_are_enabled(self):
        self.driver.implicitly_wait(0.5)
        try:
            self.driver.find_element_by_css_selector("#box0")
            self.fail("Expected NoSuchElementException to have been thrown")
        except exceptions.NoSuchElementException as e:
            pass
        except Exception as e:
            self.fail("Expected NoSuchElementException but got " + str(e))

    def test_should_return_after_first_attempt_to_find_one_after_disabling_implicit_waits(self):
        self.driver.implicitly_wait(3)
        self.driver.implicitly_wait(0)
        try:
            self.driver.find_element_by_css_selector("#box0")
            self.fail("Expected NoSuchElementException to have been thrown")
        except exceptions.NoSuchElementException as e:
            pass
        except Exception as e:
            self.fail("Expected NoSuchElementException but got " + str(e))

    def test_should_implicitly_wait_until_at_least_one_element_is_found_when_searching_for_many(self):
        add = self.driver.find_element_by_css_selector("#adder")
        self.driver.implicitly_wait(2)
        add.click()
        add.click()
        elements = self.driver.find_elements_by_css_selector(".redbox")
        self.assertTrue(len(elements) >= 1)

    def test_should_still_fail_to_find_an_element_by_class_when_implicit_waits_are_enabled(self):
        self.driver.implicitly_wait(0.5)
        elements = self.driver.find_elements_by_css_selector(".redbox")
        self.assertEqual(0, len(elements))

    def test_should_return_after_first_attempt_to_find_many_after_disabling_implicit_waits(self):
        add = self.driver.find_element_by_css_selector("#adder")
        self.driver.implicitly_wait(1.1)
        self.driver.implicitly_wait(0)
        add.click()
        elements = self.driver.find_elements_by_css_selector(".redbox")
        self.assertEqual(0, len(elements))


if __name__ == "__main__":
    unittest.main()
