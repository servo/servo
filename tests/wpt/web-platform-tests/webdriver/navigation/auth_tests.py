import os
import sys
import unittest
import ConfigParser

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test
from selenium.common import exceptions
from wptserve import server
from wptserve.router import any_method
from wptserve.handlers import basic_auth_handler

class WebDriverAuthTest(unittest.TestCase):

    # Set up class to start HTTP Server that responds to
    # test URLs with various 401 responses
    @classmethod
    def setUpClass(cls):
        cls.driver = base_test.create_driver()
        cls.webserver = server.WebTestHttpd(routes=[(any_method, "*", basic_auth_handler)])
        cls.webserver.start()

    @classmethod
    def tearDownClass(cls):
        cls.driver.quit()
        cls.webserver.stop()

    # Test that when 401 is seen by browser, a WebDriver response is still sent
    def test_response_401_auth_basic(self):
        page = self.webserver.get_url('navigation/res/authenticated.html')
        self.driver.set_page_load_timeout(5)
        try:
            self.driver.get( page )
            # if we got a responses instead of timeout, that's success
            self.assertTrue(True)
        except exceptions.TimeoutException:
            self.fail("Did not get response from browser.")
        except:
            self.fail("Unexpected failure. Please investigate.")

if __name__ == "__main__":
    unittest.main()
