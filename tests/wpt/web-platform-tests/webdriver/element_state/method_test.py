import os
import sys
import unittest

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test


class GetElementAttributeTest(base_test.WebDriverBaseTest):
    def test_get_element_attribute(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-with-id-attribute.html"))
        el = self.driver.find_element_by_css("div")
        self.assertEqual("myId", el.get_attribute("id"))

    def test_style_attribute(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-with-style-attribute.html"))
        el = self.driver.find_element_by_css("div")
        expected_style = """
                         font-family: \"Gill Sans Extrabold\",Helvetica,sans-serif;
                         line-height: 1.2; font-weight: bold;
                         """
        self.assertEqual(expected_style, el.get_attribute("style"))

    def test_color_serialization_of_style_attribute(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-with-color-style-attribute.html"))
        el = self.driver.find_element_by_css("div")
        self.assertEqual("color: rgba(255, 0, 0, 1.0);", el.get_attribute("style"))

    def test_true_if_boolean_attribute_present(self):
        self.driver.get(self.webserver.where_is("element_state/res/input-with-checked-attribute.html"))
        el = self.driver.find_element_by_css("input")
        self.assertEqual("true", el.get_attribute("checked"))

    def test_none_if_boolean_attribute_absent(self):
        self.driver.get(self.webserver.where_is("element_state/res/input-without-checked-attribute.html"))
        el = self.driver.find_element_by_css("input")
        self.assertIsNone(el.get_attribute("checked"))

    def test_option_with_attribute_value(self):
        self.driver.get(self.webserver.where_is("element_state/res/option-with-value-attribute.html"))
        el = self.driver.find_element_by_css("option")
        self.assertEqual("value1", el.get_attribute("value"))

    def test_option_without_value_attribute(self):
        self.driver.get(self.webserver.where_is("element_state/res/option-without-value-attribute.html"))
        el = self.driver.find_element_by_css("option")
        self.assertEqual("Value 1", el.get_attribute("value"))

    def test_a_href_attribute(self):
        self.driver.get(self.webserver.where_is("element_state/res/a-with-href-attribute.html"))
        el = self.driver.find_element_by_css("a")
        self.assertEqual("http://web-platform.test:8000/path#fragment", el.get_attribute("href"))

    def test_img_src_attribute(self):
        self.driver.get(self.webserver.where_is("element_state/res/img-with-src-attribute.html"))
        el = self.driver.find_element_by_css("img")
        self.assertEqual("http://web-platform.test:8000/images/blue.png", el.get_attribute("src"))

    def test_custom_attribute(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-with-custom-attribute.html"))
        el = self.driver.find_element_by_css("div")
        self.assertEqual("attribute value", el.get_attribute("webdriver-custom-attribute"))

    def test_attribute_not_present(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-without-attribute.html"))
        el = self.driver.find_element_by_css("div")
        self.assertIsNone(el.get_attribute("class"))


if __name__ == "__main__":
    unittest.main()
