import os
import unittest
import json
import types

from http.client import BadStatusLine
from io import BytesIO

import pytest

wptserve = pytest.importorskip("wptserve")
from .base import TestUsingServer, TestUsingH2Server, doc_root

def send_body_as_header(self):
    if self._response.add_required_headers:
        self.write_default_headers()

    self.write("X-Body: ")
    self._headers_complete = True

class TestResponse(TestUsingServer):
    def test_head_without_body(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.writer.end_headers = types.MethodType(send_body_as_header,
                                                           response.writer)
            return [("X-Test", "TEST")], "body\r\n"

        route = ("GET", "/test/test_head_without_body", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], method="HEAD")
        self.assertEqual("6", resp.info()['Content-Length'])
        self.assertEqual("TEST", resp.info()['x-Test'])
        self.assertEqual("", resp.info()['x-body'])

    def test_head_with_body(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.send_body_for_head_request = True
            response.writer.end_headers = types.MethodType(send_body_as_header,
                                                           response.writer)
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
        assert sorted(x.lower() for x in resp.info().keys()) == sorted(['test-header', 'date', 'server', 'content-length'])

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
            with pytest.raises(ValueError):
                response.writer.write_raw_content(None)

        route = ("GET", "/test/test_write_raw_content", handler)
        self.server.router.register(*route)
        self.request(route[1])

    def test_write_raw_contents_invalid_http(self):
        resp_content = b"INVALID HTTP"

        @wptserve.handlers.handler
        def handler(request, response):
            response.writer.write_raw_content(resp_content)

        route = ("GET", "/test/test_write_raw_content", handler)
        self.server.router.register(*route)

        with pytest.raises(BadStatusLine) as e:
            self.request(route[1])
        assert str(e.value) == resp_content.decode('utf-8')

class TestH2Response(TestUsingH2Server):
    def test_write_without_ending_stream(self):
        data = b"TEST"

        @wptserve.handlers.handler
        def handler(request, response):
            headers = [
                ('server', 'test-h2'),
                ('test', 'PASS'),
            ]
            response.writer.write_headers(headers, 202)
            response.writer.write_data_frame(data, False)

            # Should detect stream isn't ended and call `writer.end_stream()`

        route = ("GET", "/h2test/test", handler)
        self.server.router.register(*route)
        resp = self.client.get(route[1])

        assert resp.status_code == 202
        assert [x for x in resp.headers.items()] == [('server', 'test-h2'), ('test', 'PASS')]
        assert resp.content == data

    def test_set_error(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.set_error(503, message="Test error")

        route = ("GET", "/h2test/test_set_error", handler)
        self.server.router.register(*route)
        resp = self.client.get(route[1])

        assert resp.status_code == 503
        assert json.loads(resp.content) == json.loads("{\"error\": {\"message\": \"Test error\", \"code\": 503}}")

    def test_file_like_response(self):
        @wptserve.handlers.handler
        def handler(request, response):
            content = BytesIO(b"Hello, world!")
            response.content = content

        route = ("GET", "/h2test/test_file_like_response", handler)
        self.server.router.register(*route)
        resp = self.client.get(route[1])

        assert resp.status_code == 200
        assert resp.content == b"Hello, world!"

    def test_list_response(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.content = ['hello', 'world']

        route = ("GET", "/h2test/test_file_like_response", handler)
        self.server.router.register(*route)
        resp = self.client.get(route[1])

        assert resp.status_code == 200
        assert resp.content == b"helloworld"

    def test_content_longer_than_frame_size(self):
        @wptserve.handlers.handler
        def handler(request, response):
            size = response.writer.get_max_payload_size()
            content = "a" * (size + 5)
            return [('payload_size', size)], content

        route = ("GET", "/h2test/test_content_longer_than_frame_size", handler)
        self.server.router.register(*route)
        resp = self.client.get(route[1])

        assert resp.status_code == 200
        payload_size = int(resp.headers['payload_size'])
        assert payload_size
        assert resp.content == b"a" * (payload_size + 5)

    def test_encode(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.encoding = "utf8"
            t = response.writer.encode("hello")
            assert t == b"hello"

            with pytest.raises(ValueError):
                response.writer.encode(None)

        route = ("GET", "/h2test/test_content_longer_than_frame_size", handler)
        self.server.router.register(*route)
        resp = self.client.get(route[1])
        assert resp.status_code == 200

    def test_raw_header_frame(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.writer.write_raw_header_frame([
                (':status', '204'),
                ('server', 'TEST-H2')
            ], end_headers=True)

        route = ("GET", "/h2test/test_file_like_response", handler)
        self.server.router.register(*route)
        resp = self.client.get(route[1])

        assert resp.status_code == 204
        assert resp.headers['server'] == 'TEST-H2'
        assert resp.content == b''

    def test_raw_data_frame(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.write_status_headers()
            response.writer.write_raw_data_frame(data=b'Hello world', end_stream=True)

        route = ("GET", "/h2test/test_file_like_response", handler)
        self.server.router.register(*route)
        resp = self.client.get(route[1])

        assert resp.content == b'Hello world'

    def test_raw_header_continuation_frame(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.writer.write_raw_header_frame([
                (':status', '204')
            ])

            response.writer.write_raw_continuation_frame([
                ('server', 'TEST-H2')
            ], end_headers=True)

        route = ("GET", "/h2test/test_file_like_response", handler)
        self.server.router.register(*route)
        resp = self.client.get(route[1])

        assert resp.status_code == 204
        assert resp.headers['server'] == 'TEST-H2'
        assert resp.content == b''

if __name__ == '__main__':
    unittest.main()
