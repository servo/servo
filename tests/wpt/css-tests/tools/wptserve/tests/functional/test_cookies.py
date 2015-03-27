import os
import unittest
import urllib2
import json

import wptserve
from base import TestUsingServer, doc_root

class TestResponseSetCookie(TestUsingServer):
    def test_name_value(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.set_cookie("name", "value")
            return "Test"

        route = ("GET", "/test/name_value", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])

        self.assertEquals(resp.info()["Set-Cookie"], "name=value; Path=/")

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

        self.assertEquals(parts["name"], "")
        self.assertEquals(parts["Path"], "/")
        #Should also check that expires is in the past

class TestRequestCookies(TestUsingServer):
    def test_set_cookie(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return request.cookies["name"].value

        route = ("GET", "/test/set_cookie", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], headers={"Cookie": "name=value"})

        self.assertEquals(resp.read(), "value")

if __name__ == '__main__':
    unittest.main()
