import os
import unittest
import urllib2
import json
import time
from types import MethodType

import wptserve
from base import TestUsingServer, doc_root

def send_body_as_header(self):
    if self._response.add_required_headers:
        self.write_default_headers()

    self.write("X-Body: ")
    self._headers_complete = True

class TestResponse(TestUsingServer):
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
        self.assertEquals("6", resp.info()['Content-Length'])
        self.assertEquals("TEST", resp.info()['x-Test'])
        self.assertEquals("", resp.info()['x-body'])

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
        self.assertEquals("6", resp.info()['Content-Length'])
        self.assertEquals("TEST", resp.info()['x-Test'])
        self.assertEquals("body", resp.info()['X-Body'])

if __name__ == '__main__':
    unittest.main()
