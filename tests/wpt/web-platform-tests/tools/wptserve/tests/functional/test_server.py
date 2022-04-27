import unittest

import pytest
from urllib.error import HTTPError

wptserve = pytest.importorskip("wptserve")
from .base import TestUsingServer, TestUsingH2Server


class TestFileHandler(TestUsingServer):
    def test_not_handled(self):
        with self.assertRaises(HTTPError) as cm:
            self.request("/not_existing")

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
        self.assertEqual(b"/test/rewritten", resp.read())


class TestRequestHandler(TestUsingServer):
    def test_exception(self):
        @wptserve.handlers.handler
        def handler(request, response):
            raise Exception

        route = ("GET", "/test/raises", handler)
        self.server.router.register(*route)
        with self.assertRaises(HTTPError) as cm:
            self.request("/test/raises")

        self.assertEqual(cm.exception.code, 500)

    def test_many_headers(self):
        headers = {"X-Val%d" % i: str(i) for i in range(256)}

        @wptserve.handlers.handler
        def handler(request, response):
            # Additional headers are added by urllib.request.
            assert len(request.headers) > len(headers)
            for k, v in headers.items():
                assert request.headers.get(k) == \
                    wptserve.utils.isomorphic_encode(v)
            return "OK"

        route = ("GET", "/test/headers", handler)
        self.server.router.register(*route)
        resp = self.request("/test/headers", headers=headers)
        self.assertEqual(200, resp.getcode())


class TestH2Version(TestUsingH2Server):
    # The purpose of this test is to ensure that all TestUsingH2Server tests
    # actually end up using HTTP/2, in case there's any protocol negotiation.
    def test_http_version(self):
        resp = self.client.get('/')

        assert resp.http_version == 'HTTP/2'


class TestFileHandlerH2(TestUsingH2Server):
    def test_not_handled(self):
        resp = self.client.get("/not_existing")

        assert resp.status_code == 404


class TestRewriterH2(TestUsingH2Server):
    def test_rewrite(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return request.request_path

        route = ("GET", "/test/rewritten", handler)
        self.server.rewriter.register("GET", "/test/original", route[1])
        self.server.router.register(*route)
        resp = self.client.get("/test/original")
        assert resp.status_code == 200
        assert resp.content == b"/test/rewritten"


class TestRequestHandlerH2(TestUsingH2Server):
    def test_exception(self):
        @wptserve.handlers.handler
        def handler(request, response):
            raise Exception

        route = ("GET", "/test/raises", handler)
        self.server.router.register(*route)
        resp = self.client.get("/test/raises")

        assert resp.status_code == 500

    def test_frame_handler_exception(self):
        class handler_cls:
            def frame_handler(self, request):
                raise Exception

        route = ("GET", "/test/raises", handler_cls())
        self.server.router.register(*route)
        resp = self.client.get("/test/raises")

        assert resp.status_code == 500


if __name__ == "__main__":
    unittest.main()
