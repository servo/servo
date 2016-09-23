import unittest
import urllib2

import wptserve
from .base import TestUsingServer

class TestFileHandler(TestUsingServer):
    def test_not_handled(self):
        with self.assertRaises(urllib2.HTTPError) as cm:
            resp = self.request("/not_existing")

        self.assertEqual(cm.exception.code, 404)

class TestRewriter(TestUsingServer):
    def test_rewrite(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return request.request_path

        route = ("GET", "/test/rewritten", handler)
        self.server.rewriter.register("GET", "/test/original", route[1])
        self.server.router.register(*route)
        resp = self.request("/test/original")
        self.assertEqual(200, resp.getcode())
        self.assertEqual("/test/rewritten", resp.read())

class TestRequestHandler(TestUsingServer):
    def test_exception(self):
        @wptserve.handlers.handler
        def handler(request, response):
            raise Exception

        route = ("GET", "/test/raises", handler)
        self.server.router.register(*route)
        with self.assertRaises(urllib2.HTTPError) as cm:
            resp = self.request("/test/raises")

        self.assertEqual(cm.exception.code, 500)

if __name__ == "__main__":
    unittest.main()
