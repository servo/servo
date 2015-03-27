import BaseHTTPServer
import os
import ssl
import sys
import unittest

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test


here = os.path.dirname(os.path.abspath(__file__))


class InvalidCertTest(base_test.WebDriverBaseTest):
    def testCanNavigateToSiteWithSelfSignedCert(self):
        self.webserver.httpd.socket = ssl.wrap_socket(
            self.webserver.httpd.socket,
            certfile=os.path.join(here, 'res/self-signed.key'),
            server_side=True)
        expected = self.webserver.where_is(
            'navigation/res/empty.html').replace('http:', 'https:', 1)

        self.driver.get(expected)
        self.assertEquals(expected, self.driver.get_current_url())


if __name__ == '__main__':
    unittest.main()
