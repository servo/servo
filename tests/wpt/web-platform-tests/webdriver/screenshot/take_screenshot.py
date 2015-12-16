import os
import sys
import unittest

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test


class ScreenShotTest(base_test.WebDriverBaseTest):
    # Get a static page that must be the same upon refresh
    def test_screenShot(self):
        self.driver.get(self.webserver.where_is('screenshot/res/screen.html'))

if __name__ == '__main__':
    unittest.main()
