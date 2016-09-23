import unittest

import wptserve
from .base import TestUsingServer

class TestInputFile(TestUsingServer):
    def test_seek(self):
        @wptserve.handlers.handler
        def handler(request, response):
            rv = []
            f = request.raw_input
            f.seek(5)
            rv.append(f.read(2))
            rv.append(f.tell())
            f.seek(0)
            rv.append(f.readline())
            rv.append(f.tell())
            rv.append(f.read(-1))
            rv.append(f.tell())
            f.seek(0)
            rv.append(f.read())
            f.seek(0)
            rv.extend(f.readlines())

            return " ".join(str(item) for item in rv)

        route = ("POST", "/test/test_seek", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], method="POST", body="12345ab\ncdef")
        self.assertEqual(200, resp.getcode())
        self.assertEqual(["ab", "7", "12345ab\n", "8", "cdef", "12",
                          "12345ab\ncdef", "12345ab\n", "cdef"],
                         resp.read().split(" "))

    def test_iter(self):
        @wptserve.handlers.handler
        def handler(request, response):
            f = request.raw_input
            return " ".join(line for line in f)

        route = ("POST", "/test/test_iter", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], method="POST", body="12345\nabcdef\r\nzyxwv")
        self.assertEqual(200, resp.getcode())
        self.assertEqual(["12345\n", "abcdef\r\n", "zyxwv"], resp.read().split(" "))

class TestRequest(TestUsingServer):
    def test_body(self):
        @wptserve.handlers.handler
        def handler(request, response):
            request.raw_input.seek(5)
            return request.body

        route = ("POST", "/test/test_body", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], method="POST", body="12345ab\ncdef")
        self.assertEqual("12345ab\ncdef", resp.read())

    def test_route_match(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return request.route_match["match"] + " " + request.route_match["*"]

        route = ("GET", "/test/{match}_*", handler)
        self.server.router.register(*route)
        resp = self.request("/test/some_route")
        self.assertEqual("some route", resp.read())

class TestAuth(TestUsingServer):
    def test_auth(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return " ".join((request.auth.username, request.auth.password))

        route = ("GET", "/test/test_auth", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], auth=("test", "PASS"))
        self.assertEqual(200, resp.getcode())
        self.assertEqual(["test", "PASS"], resp.read().split(" "))

if __name__ == '__main__':
    unittest.main()
