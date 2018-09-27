import pytest

wptserve = pytest.importorskip("wptserve")
from .base import TestUsingServer
from wptserve.request import InputFile


class TestInputFile(TestUsingServer):
    def test_seek(self):
        @wptserve.handlers.handler
        def handler(request, response):
            rv = []
            f = request.raw_input
            f.seek(5)
            rv.append(f.read(2))
            rv.append(b"%d" % f.tell())
            f.seek(0)
            rv.append(f.readline())
            rv.append(b"%d" % f.tell())
            rv.append(f.read(-1))
            rv.append(b"%d" % f.tell())
            f.seek(0)
            rv.append(f.read())
            f.seek(0)
            rv.extend(f.readlines())

            return b" ".join(rv)

        route = ("POST", "/test/test_seek", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], method="POST", body=b"12345ab\ncdef")
        self.assertEqual(200, resp.getcode())
        self.assertEqual([b"ab", b"7", b"12345ab\n", b"8", b"cdef", b"12",
                          b"12345ab\ncdef", b"12345ab\n", b"cdef"],
                         resp.read().split(b" "))

    def test_seek_input_longer_than_buffer(self):
        @wptserve.handlers.handler
        def handler(request, response):
            rv = []
            f = request.raw_input
            f.seek(5)
            rv.append(f.read(2))
            rv.append(b"%d" % f.tell())
            f.seek(0)
            rv.append(b"%d" % f.tell())
            rv.append(b"%d" % f.tell())
            return b" ".join(rv)

        route = ("POST", "/test/test_seek", handler)
        self.server.router.register(*route)

        old_max_buf = InputFile.max_buffer_size
        InputFile.max_buffer_size = 10
        try:
            resp = self.request(route[1], method="POST", body=b"1"*20)
            self.assertEqual(200, resp.getcode())
            self.assertEqual([b"11", b"7", b"0", b"0"],
                             resp.read().split(b" "))
        finally:
            InputFile.max_buffer_size = old_max_buf

    def test_iter(self):
        @wptserve.handlers.handler
        def handler(request, response):
            f = request.raw_input
            return b" ".join(line for line in f)

        route = ("POST", "/test/test_iter", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], method="POST", body=b"12345\nabcdef\r\nzyxwv")
        self.assertEqual(200, resp.getcode())
        self.assertEqual([b"12345\n", b"abcdef\r\n", b"zyxwv"], resp.read().split(b" "))

    def test_iter_input_longer_than_buffer(self):
        @wptserve.handlers.handler
        def handler(request, response):
            f = request.raw_input
            return b" ".join(line for line in f)

        route = ("POST", "/test/test_iter", handler)
        self.server.router.register(*route)

        old_max_buf = InputFile.max_buffer_size
        InputFile.max_buffer_size = 10
        try:
            resp = self.request(route[1], method="POST", body=b"12345\nabcdef\r\nzyxwv")
            self.assertEqual(200, resp.getcode())
            self.assertEqual([b"12345\n", b"abcdef\r\n", b"zyxwv"], resp.read().split(b" "))
        finally:
            InputFile.max_buffer_size = old_max_buf


class TestRequest(TestUsingServer):
    def test_body(self):
        @wptserve.handlers.handler
        def handler(request, response):
            request.raw_input.seek(5)
            return request.body

        route = ("POST", "/test/test_body", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], method="POST", body=b"12345ab\ncdef")
        self.assertEqual(b"12345ab\ncdef", resp.read())

    def test_route_match(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return request.route_match["match"] + " " + request.route_match["*"]

        route = ("GET", "/test/{match}_*", handler)
        self.server.router.register(*route)
        resp = self.request("/test/some_route")
        self.assertEqual(b"some route", resp.read())


class TestAuth(TestUsingServer):
    def test_auth(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return " ".join((request.auth.username, request.auth.password))

        route = ("GET", "/test/test_auth", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], auth=("test", "PASS"))
        self.assertEqual(200, resp.getcode())
        self.assertEqual([b"test", b"PASS"], resp.read().split(b" "))
