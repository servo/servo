import os
import sys
import random
import unittest

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test

repo_root = os.path.abspath(os.path.join(__file__, "../../.."))
sys.path.insert(1, os.path.join(repo_root, "tools", "webdriver"))
from webdriver import exceptions


class OpenAndCloseWindowTest(base_test.WebDriverBaseTest):
    def setUp(self):
        self.driver.get(self.webserver.where_is("command_contexts/res/first-page.html"))

    def tearDown(self):
        handles = self.driver.get_window_handles()

        for i in range(len(handles) - 1):
            self.driver.switch_to_window(handles[i])
            self.driver.close()

        self.driver.switch_to_window(self.driver.get_window_handles()[0])

    def test_open_new_window(self):
        handles = self.driver.get_window_handles()
        self.driver.find_element_by_id("open_new_window").click()
        self.assertEquals(len(handles) + 1, len(self.driver.get_window_handles()))

    def test_get_window_handles_returns_the_windows_that_have_been_opened(self):
        self.driver.find_element_by_id("open_new_window").click()
        handles = self.driver.get_window_handles()
        self.driver.switch_to_window(handles[0])
        url1 = self.driver.get_current_url()
        self.driver.switch_to_window(handles[1])
        url2 = self.driver.get_current_url()

        if url1 == self.webserver.where_is("controlling_windows/res/other-page.html"):
            self.assertEquals(url2, self.webserver.where_is("controlling_windows/res/first-page.html"))
        elif url1 == self.webserver.where_is("controlling_windows/res/first-page.html"):
            self.assertEquals(url2, self.webserver.where_is("controlling_windows/res/other-page.html"))
        else:
            self.fail("The wrong set of URLs were returned")

    def test_close_window(self):
        open_windows = len(self.driver.get_window_handles())

        self.driver.find_element_by_id("open_new_window").click()
        self.assertEquals(1 + open_windows, len(self.driver.get_window_handles()))

        self.driver.close()
        self.assertEquals(open_windows, len(self.driver.get_window_handles()))

    def test_command_sent_to_closed_window_returns_no_such_window_exception(self):
        self.driver.find_element_by_id("open_new_window").click()
        self.driver.close()

        with self.assertRaises(exceptions.NoSuchWindowException):
            self.driver.get_window_handle()

if __name__ == "__main__":
    unittest.main()
