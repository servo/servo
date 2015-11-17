import os
import sys
import unittest

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test

repo_root = os.path.abspath(os.path.join(__file__, "../../.."))
sys.path.insert(1, os.path.join(repo_root, "tools", "webdriver"))
from webdriver import exceptions, wait


class ClickTest(base_test.WebDriverBaseTest):
    def setUp(self):
        self.wait = wait.WebDriverWait(self.driver, 5, ignored_exceptions = [exceptions.NoSuchAlertException])
        self.driver.get(self.webserver.where_is('modal/res/alerts.html'))

    def tearDown(self):
        try:
            self.driver.switch_to_alert().dismiss()
        except exceptions.NoSuchAlertException:
            pass

    def test_click_div(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("div")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "div")

    def test_click_p(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("p")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "p")

    def test_click_h1(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("h1")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "h1")

    def test_click_pre(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("pre")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "pre")

    def test_click_ol(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("ol")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "ol")

    def test_click_ul(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("ul")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "ul")

    def test_click_a(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("a")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "a")

    def test_click_img(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("img")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "img")

    def test_click_video(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("video")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "video")

    def test_click_canvas(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("canvas")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "canvas")

    def test_click_progress(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("progress")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "progress")

    def test_click_textarea(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("textarea")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "textarea")

    def test_click_button(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("button")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "button")

    def test_click_svg(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("svg")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "svg")

    def test_click_input_range(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("input_range")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "input_range")

    def test_click_input_button(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("input_button")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "input_button")

    def test_click_input_submit(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("input_submit")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "input_submit")

    def test_click_input_reset(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("input_reset")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "input_reset")

    def test_click_input_checkbox(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("input_checkbox")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "input_checkbox")

    def test_click_input_radio(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("input_radio")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "input_radio")

    def test_click_input_text(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("input_text")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "input_text")

    def test_click_input_number(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("input_number")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "input_number")

    def test_click_input_tel(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("input_tel")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "input_tel")

    def test_click_input_url(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("input_url")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "input_url")

    def test_click_input_email(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("input_email")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "input_email")

    def test_click_input_search(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("input_search")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "input_search")

    def test_click_input_image(self):
        self.driver.get(self.webserver.where_is("user_input/res/click.html"))

        element = self.driver.find_element_by_id("input_image")
        element.click()

        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.get_text()
        alert.accept()

        self.assertEquals(value, "input_image")

if __name__ == "__main__":
    unittest.main()
