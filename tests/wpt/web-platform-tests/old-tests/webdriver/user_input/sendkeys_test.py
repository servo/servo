import os
import sys
import random
import unittest

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test

repo_root = os.path.abspath(os.path.join(__file__, "../../.."))
sys.path.insert(1, os.path.join(repo_root, "tools", "webdriver"))
from webdriver import exceptions


class SendKeysTest(base_test.WebDriverBaseTest):
    def setUp(self):
        self.driver.get(self.webserver.where_is("user_input/res/text-form.html"))

    def test_send_simple_string(self):
        element = self.driver.find_element_by_id("Text1")
        element.send_keys("lorem ipsum")

        self.assertEquals(self.driver.find_element_by_id("text").get_text(), u"lorem ipsum")

    def test_send_return(self):
        element = self.driver.find_element_by_id("Text1")
        returnkey = unichr(int("E006", 16))
        element.send_keys([returnkey])

        self.assertEquals(u"" + self.driver.get_current_url(), u"" + self.webserver.where_is("user_input/res/text-form-landing.html?e=mc2"))

    def test_send_backspace(self):
        element = self.driver.find_element_by_id("Text1")
        element.send_keys("world ")
        element.send_keys("wide ")
        element.send_keys("web ")
        element.send_keys("consortium")

        backspace= unichr(int("E003", 16))
        for i in range(0, 11):
            element.send_keys([backspace])

        self.assertEquals(self.driver.find_element_by_id("text").get_text(), u"world wide web")

    def test_send_tab(self):
        element1 = self.driver.find_element_by_id("Text1")
        element2 = self.driver.find_element_by_id("Text2")
        element1.send_keys("typing here")

        tab= unichr(int("E004", 16))
        element1.send_keys([tab])

        output = self.driver.find_element_by_id("output")
        tab_pressed = output.get_attribute("checked")
        self.assertEquals(tab_pressed, u"true")

    def test_send_shift(self):
        element = self.driver.find_element_by_id("Text1")
        element.send_keys("low ")

        shift= unichr(int("E008", 16))
        element.send_keys([shift , "u", "p", shift])

        self.assertEquals(self.driver.find_element_by_id("text").get_text(), u"low UP")

    def test_send_arrow_keys(self):
        element = self.driver.find_element_by_id("Text1")

        element.send_keys("internet")

        backspace= unichr(int("E003", 16))
        left= unichr(int("E012", 16))
        right= unichr(int("E014", 16))
        for i in range(0, 4):
            element.send_keys([left])

        element.send_keys([backspace])
        element.send_keys([right])
        element.send_keys("a")

        self.assertEquals(self.driver.find_element_by_id("text").get_text(), u"intranet")

    def test_select_text_with_shift(self):
        element = self.driver.find_element_by_id("Text1")

        element.send_keys("WebDriver")
        backspace= unichr(int("E003", 16))
        shift= unichr(int("E008", 16))
        left= unichr(int("E012", 16))

        element.send_keys([shift, left, left, left, left, left, left, backspace])

        self.assertEquals(self.driver.find_element_by_id("text").get_text(), u"Web")


if __name__ == "__main__":
    unittest.main()
