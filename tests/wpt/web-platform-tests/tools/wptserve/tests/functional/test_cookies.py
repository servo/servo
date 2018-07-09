import sys
import unittest

import pytest

wptserve = pytest.importorskip("wptserve")
from .base import TestUsingServer


class TestResponseSetCookie(TestUsingServer):
    @pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
    def test_name_value(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.set_cookie("name", "value")
            return "Test"

        route = ("GET", "/test/name_value", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])

        self.assertEqual(resp.info()["Set-Cookie"], "name=value; Path=/")

    @pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
    def test_unset(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.set_cookie("name", "value")
            response.unset_cookie("name")
            return "Test"

        route = ("GET", "/test/unset", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])

        self.assertTrue("Set-Cookie" not in resp.info())

    @pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
    def test_delete(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.delete_cookie("name")
            return "Test"

        route = ("GET", "/test/delete", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])

        parts = dict(item.split("=") for
                     item in resp.info()["Set-Cookie"].split("; ") if item)

        self.assertEqual(parts["name"], "")
        self.assertEqual(parts["Path"], "/")
        #Should also check that expires is in the past

class TestRequestCookies(TestUsingServer):
    @pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
    def test_set_cookie(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return request.cookies["name"].value

        route = ("GET", "/test/set_cookie", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], headers={"Cookie": "name=value"})
        self.assertEqual(resp.read(), b"value")

if __name__ == '__main__':
    unittest.main()
