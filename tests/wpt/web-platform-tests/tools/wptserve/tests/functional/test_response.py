import sys
import unittest
from types import MethodType

import pytest

wptserve = pytest.importorskip("wptserve")
from .base import TestUsingServer


def send_body_as_header(self):
    if self._response.add_required_headers:
        self.write_default_headers()

    self.write("X-Body: ")
    self._headers_complete = True

class TestResponse(TestUsingServer):
    @pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
    def test_head_without_body(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.writer.end_headers = MethodType(send_body_as_header,
                                                     response.writer,
                                                     wptserve.response.ResponseWriter)
            return [("X-Test", "TEST")], "body\r\n"

        route = ("GET", "/test/test_head_without_body", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], method="HEAD")
        self.assertEqual("6", resp.info()['Content-Length'])
        self.assertEqual("TEST", resp.info()['x-Test'])
        self.assertEqual("", resp.info()['x-body'])

    @pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
    def test_head_with_body(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.send_body_for_head_request = True
            response.writer.end_headers = MethodType(send_body_as_header,
                                                     response.writer,
                                                     wptserve.response.ResponseWriter)
            return [("X-Test", "TEST")], "body\r\n"

        route = ("GET", "/test/test_head_with_body", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], method="HEAD")
        self.assertEqual("6", resp.info()['Content-Length'])
        self.assertEqual("TEST", resp.info()['x-Test'])
        self.assertEqual("body", resp.info()['X-Body'])

if __name__ == '__main__':
    unittest.main()
