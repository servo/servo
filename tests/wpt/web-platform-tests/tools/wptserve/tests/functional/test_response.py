import sys
import os
import unittest
from types import MethodType

import pytest

wptserve = pytest.importorskip("wptserve")
from .base import TestUsingServer, doc_root


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

    def test_write_content_no_status_no_header(self):
        resp_content = b"TEST"

        @wptserve.handlers.handler
        def handler(request, response):
            response.writer.write_content(resp_content)

        route = ("GET", "/test/test_write_content_no_status_no_header", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])
        assert resp.getcode() == 200
        assert resp.read() == resp_content
        assert resp.info()["Content-Length"] == str(len(resp_content))
        assert "Date" in resp.info()
        assert "Server" in resp.info()

    def test_write_content_no_headers(self):
        resp_content = b"TEST"

        @wptserve.handlers.handler
        def handler(request, response):
            response.writer.write_status(201)
            response.writer.write_content(resp_content)

        route = ("GET", "/test/test_write_content_no_headers", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])
        assert resp.getcode() == 201
        assert resp.read() == resp_content
        assert resp.info()["Content-Length"] == str(len(resp_content))
        assert "Date" in resp.info()
        assert "Server" in resp.info()

    def test_write_content_no_status(self):
        resp_content = b"TEST"

        @wptserve.handlers.handler
        def handler(request, response):
            response.writer.write_header("test-header", "test-value")
            response.writer.write_content(resp_content)

        route = ("GET", "/test/test_write_content_no_status", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])
        assert resp.getcode() == 200
        assert resp.read() == resp_content
        assert sorted([x.lower() for x in resp.info().keys()]) == sorted(['test-header', 'date', 'server', 'content-length'])

    def test_write_content_no_status_no_required_headers(self):
        resp_content = b"TEST"

        @wptserve.handlers.handler
        def handler(request, response):
            response.add_required_headers = False
            response.writer.write_header("test-header", "test-value")
            response.writer.write_content(resp_content)

        route = ("GET", "/test/test_write_content_no_status_no_required_headers", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])
        assert resp.getcode() == 200
        assert resp.read() == resp_content
        assert resp.info().items() == [('test-header', 'test-value')]

    def test_write_content_no_status_no_headers_no_required_headers(self):
        resp_content = b"TEST"

        @wptserve.handlers.handler
        def handler(request, response):
            response.add_required_headers = False
            response.writer.write_content(resp_content)

        route = ("GET", "/test/test_write_content_no_status_no_headers_no_required_headers", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])
        assert resp.getcode() == 200
        assert resp.read() == resp_content
        assert resp.info().items() == []

    def test_write_raw_content(self):
        resp_content = b"HTTP/1.1 202 Giraffe\n" \
            b"X-TEST: PASS\n" \
            b"Content-Length: 7\n\n" \
            b"Content"

        @wptserve.handlers.handler
        def handler(request, response):
            response.writer.write_raw_content(resp_content)

        route = ("GET", "/test/test_write_raw_content", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])
        assert resp.getcode() == 202
        assert resp.info()["X-TEST"] == "PASS"
        assert resp.read() == b"Content"

    def test_write_raw_content_file(self):
        @wptserve.handlers.handler
        def handler(request, response):
            with open(os.path.join(doc_root, "test.asis"), 'rb') as infile:
                response.writer.write_raw_content(infile)

        route = ("GET", "/test/test_write_raw_content", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])
        assert resp.getcode() == 202
        assert resp.info()["X-TEST"] == "PASS"
        assert resp.read() == b"Content"

    def test_write_raw_none(self):
        @wptserve.handlers.handler
        def handler(request, response):
            with pytest.raises(ValueError, message="data cannot be None"):
                response.writer.write_raw_content(None)

        route = ("GET", "/test/test_write_raw_content", handler)
        self.server.router.register(*route)
        self.request(route[1])

    @pytest.mark.xfail(sys.version_info >= (3,), reason="py3 urllib doesn't handle invalid HTTP very well")
    def test_write_raw_contents_invalid_http(self):
        resp_content = b"INVALID HTTP"

        @wptserve.handlers.handler
        def handler(request, response):
            response.writer.write_raw_content(resp_content)

        route = ("GET", "/test/test_write_raw_content", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])
        assert resp.read() == resp_content

if __name__ == '__main__':
    unittest.main()
