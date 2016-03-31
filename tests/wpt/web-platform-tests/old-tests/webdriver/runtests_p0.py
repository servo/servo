import unittest

from unittest import TestLoader, TextTestRunner, TestSuite

from cookie import cookie_test
from navigation import forward
from navigation import forwardToNothing
from navigation import get_from_http_test
from navigation import refresh_page
from element_location import element_location_test
from element_state import visibility_test
from element_state import method_test
from element_state import properties
from javascript import execute_script_test
from user_input import clear_test
from windows import window_manipulation
from windows import tabbing



if __name__ == "__main__":

    loader = TestLoader()
    suite = TestSuite((
        loader.loadTestsFromModule(cookie_test),
        loader.loadTestsFromModule(forward),
        loader.loadTestsFromModule(forwardToNothing),
        loader.loadTestsFromModule(element_location_test),
        loader.loadTestsFromModule(visibility_test),
        loader.loadTestsFromModule(execute_script_test),
        loader.loadTestsFromModule(clear_test),
        loader.loadTestsFromModule(method_test),
        loader.loadTestsFromModule(properties),
        loader.loadTestsFromModule(refresh_page),
        loader.loadTestsFromModule(get_from_http_test),
        loader.loadTestsFromModule(window_manipulation),
        loader.loadTestsFromModule(tabbing)
        ))

    runner = TextTestRunner(verbosity=2)
    runner.run(suite)
    unittest.main()
