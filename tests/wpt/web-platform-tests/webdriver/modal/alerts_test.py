import os
import sys
import unittest

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test
from selenium.common import exceptions
from selenium.webdriver.support import wait

class AlertsTest(base_test.WebDriverBaseTest):
    def setUp(self):
        self.wait = wait.WebDriverWait(self.driver, 5, ignored_exceptions = [exceptions.NoAlertPresentException])
        self.driver.get(self.webserver.where_is('modal/res/alerts.html'))

    def tearDown(self):
        try:
            self.driver.switch_to_alert().dismiss()
        except exceptions.NoAlertPresentException:
            pass

    # Alerts
    def test_should_allow_user_to_accept_an_alert(self):
        self.driver.find_element_by_css_selector('#alert').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        alert.accept()
        self.driver.current_url

    def test_should_allow_user_to_accept_an_alert_with_no_text(self):
        self.driver.find_element_by_css_selector('#empty-alert').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        alert.accept()
        self.driver.current_url

    def test_should_allow_user_to_dismiss_an_alert(self):
        self.driver.find_element_by_css_selector('#alert').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        alert.dismiss()
        self.driver.current_url

    def test_should_allow_user_to_get_text_of_an_alert(self):
        self.driver.find_element_by_css_selector('#alert').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.text
        alert.accept()
        self.assertEquals('cheese', value)

    def test_setting_the_value_of_an_alert_throws(self):
        self.driver.find_element_by_css_selector('#alert').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        with self.assertRaises(exceptions.ElementNotVisibleException):
            alert.send_keys('cheese')
        alert.accept()

    def test_alert_should_not_allow_additional_commands_if_dismissed(self):
        self.driver.find_element_by_css_selector('#alert').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        alert.accept()
        with self.assertRaises(exceptions.NoAlertPresentException):
            alert.text

    # Prompts
    def test_should_allow_user_to_accept_a_prompt(self):
        self.driver.find_element_by_css_selector('#prompt').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        alert.accept()
        self.wait.until(lambda x: x.find_element_by_css_selector('#text').text == '')

    def test_should_allow_user_to_dismiss_a_prompt(self):
        self.driver.find_element_by_css_selector('#prompt').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        alert.dismiss()
        self.wait.until(lambda x: x.find_element_by_css_selector('#text').text == 'null')

    def test_should_allow_user_to_set_the_value_of_a_prompt(self):
        self.driver.find_element_by_css_selector('#prompt').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        alert.send_keys('cheese')
        alert.accept()
        self.wait.until(lambda x: x.find_element_by_css_selector('#text').text == 'cheese')

    def test_should_allow_user_to_get_text_of_a_prompt(self):
        self.driver.find_element_by_css_selector('#prompt').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.text
        alert.accept()
        self.assertEquals('Enter something', value)

    def test_prompt_should_not_allow_additional_commands_if_dismissed(self):
        self.driver.find_element_by_css_selector('#prompt').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        alert.accept()
        with self.assertRaises(exceptions.NoAlertPresentException):
            alert.text

    def test_prompt_should_use_default_value_if_no_keys_sent(self):
        self.driver.find_element_by_css_selector('#prompt-with-default').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        alert.accept()
        self.wait.until(lambda x: x.find_element_by_css_selector('#text').text == 'This is a default value')

    def test_prompt_should_have_null_value_if_dismissed(self):
        self.driver.find_element_by_css_selector('#prompt-with-default').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        alert.dismiss()
        self.wait.until(lambda x: x.find_element_by_css_selector('#text').text == 'null')

    # Confirmations
    def test_should_allow_user_to_accept_a_confirm(self):
        self.driver.find_element_by_css_selector('#confirm').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        alert.accept()
        self.wait.until(lambda x: x.find_element_by_css_selector('#text').text == 'true')

    def test_should_allow_user_to_dismiss_a_confirm(self):
        self.driver.find_element_by_css_selector('#confirm').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        alert.dismiss()
        self.wait.until(lambda x: x.find_element_by_css_selector('#text').text == 'false')

    def test_setting_the_value_of_a_confirm_throws(self):
        self.driver.find_element_by_css_selector('#confirm').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        with self.assertRaises(exceptions.ElementNotVisibleException):
            alert.send_keys('cheese')
        alert.accept()

    def test_should_allow_user_to_get_text_of_a_confirm(self):
        self.driver.find_element_by_css_selector('#confirm').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        value = alert.text
        alert.accept()
        self.assertEquals('cheese', value)

    def test_confirm_should_not_allow_additional_commands_if_dismissed(self):
        self.driver.find_element_by_css_selector('#confirm').click()
        alert = self.wait.until(lambda x: x.switch_to_alert())
        alert.accept()
        with self.assertRaises(exceptions.NoAlertPresentException):
            alert.text

"""
    def test_switch_to_missing_alert_fails(self):
        with self.assertRaises(exceptions.NoAlertPresentException):
            self.driver.switch_to_alert()
"""

if __name__ == '__main__':
    unittest.main()
