import unittest

import pytest

wptserve = pytest.importorskip("wptserve")
from .base import TestUsingServer


class TestResponseSetCookie(TestUsingServer):
    def test_name_value(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.set_cookie(b"name", b"value")
            return "Test"

        route = ("GET", "/test/name_value", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])

        self.assertEqual(resp.info()["Set-Cookie"], "name=value; Path=/")

    def test_unset(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.set_cookie(b"name", b"value")
            response.unset_cookie(b"name")
            return "Test"

        route = ("GET", "/test/unset", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])

        self.assertTrue("Set-Cookie" not in resp.info())

    def test_delete(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.delete_cookie(b"name")
            return "Test"

        route = ("GET", "/test/delete", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])

        parts = dict(item.split("=") for
                     item in resp.info()["Set-Cookie"].split("; ") if item)

        self.assertEqual(parts["name"], "")
        self.assertEqual(parts["Path"], "/")
        # TODO: Should also check that expires is in the past


class TestRequestCookies(TestUsingServer):
    def test_set_cookie(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return request.cookies[b"name"].value

        route = ("GET", "/test/set_cookie", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], headers={"Cookie": "name=value"})
        self.assertEqual(resp.read(), b"value")


if __name__ == '__main__':
    unittest.main()
