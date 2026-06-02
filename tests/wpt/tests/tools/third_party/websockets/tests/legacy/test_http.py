import asyncio

from websockets.exceptions import SecurityError
from websockets.legacy.http import *
from websockets.legacy.http import read_headers

from .utils import AsyncioTestCase


class HTTPAsyncTests(AsyncioTestCase):
    def setUp(self):
        super().setUp()
        self.stream = asyncio.StreamReader(loop=self.loop)

    async def test_read_request(self):
        # Example from the protocol overview in RFC 6455
        self.stream.feed_data(
            b"GET /chat HTTP/1.1\r\n"
            b"Host: server.example.com\r\n"
            b"Upgrade: websocket\r\n"
            b"Connection: Upgrade\r\n"
            b"Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n"
            b"Origin: http://example.com\r\n"
            b"Sec-WebSocket-Protocol: chat, superchat\r\n"
            b"Sec-WebSocket-Version: 13\r\n"
            b"\r\n"
        )
        path, headers = await read_request(self.stream)
        self.assertEqual(path, "/chat")
        self.assertEqual(headers["Upgrade"], "websocket")

    async def test_read_request_empty(self):
        self.stream.feed_eof()
        with self.assertRaisesRegex(
            EOFError, "connection closed while reading HTTP request line"
        ):
            await read_request(self.stream)

    async def test_read_request_invalid_request_line(self):
        self.stream.feed_data(b"GET /\r\n\r\n")
        with self.assertRaisesRegex(ValueError, "invalid HTTP request line: GET /"):
            await read_request(self.stream)

    async def test_read_request_unsupported_method(self):
        self.stream.feed_data(b"OPTIONS * HTTP/1.1\r\n\r\n")
        with self.assertRaisesRegex(ValueError, "unsupported HTTP method: OPTIONS"):
            await read_request(self.stream)

    async def test_read_request_unsupported_version(self):
        self.stream.feed_data(b"GET /chat HTTP/1.0\r\n\r\n")
        with self.assertRaisesRegex(ValueError, "unsupported HTTP version: HTTP/1.0"):
            await read_request(self.stream)

    async def test_read_request_invalid_header(self):
        self.stream.feed_data(b"GET /chat HTTP/1.1\r\nOops\r\n")
        with self.assertRaisesRegex(ValueError, "invalid HTTP header line: Oops"):
            await read_request(self.stream)

    async def test_read_response(self):
        # Example from the protocol overview in RFC 6455
        self.stream.feed_data(
            b"HTTP/1.1 101 Switching Protocols\r\n"
            b"Upgrade: websocket\r\n"
            b"Connection: Upgrade\r\n"
            b"Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n"
            b"Sec-WebSocket-Protocol: chat\r\n"
            b"\r\n"
        )
        status_code, reason, headers = await read_response(self.stream)
        self.assertEqual(status_code, 101)
        self.assertEqual(reason, "Switching Protocols")
        self.assertEqual(headers["Upgrade"], "websocket")

    async def test_read_response_empty(self):
        self.stream.feed_eof()
        with self.assertRaisesRegex(
            EOFError, "connection closed while reading HTTP status line"
        ):
            await read_response(self.stream)

    async def test_read_request_invalid_status_line(self):
        self.stream.feed_data(b"Hello!\r\n")
        with self.assertRaisesRegex(ValueError, "invalid HTTP status line: Hello!"):
            await read_response(self.stream)

    async def test_read_response_unsupported_version(self):
        self.stream.feed_data(b"HTTP/1.0 400 Bad Request\r\n\r\n")
        with self.assertRaisesRegex(ValueError, "unsupported HTTP version: HTTP/1.0"):
            await read_response(self.stream)

    async def test_read_response_invalid_status(self):
        self.stream.feed_data(b"HTTP/1.1 OMG WTF\r\n\r\n")
        with self.assertRaisesRegex(ValueError, "invalid HTTP status code: OMG"):
            await read_response(self.stream)

    async def test_read_response_unsupported_status(self):
        self.stream.feed_data(b"HTTP/1.1 007 My name is Bond\r\n\r\n")
        with self.assertRaisesRegex(ValueError, "unsupported HTTP status code: 007"):
            await read_response(self.stream)

    async def test_read_response_invalid_reason(self):
        self.stream.feed_data(b"HTTP/1.1 200 \x7f\r\n\r\n")
        with self.assertRaisesRegex(ValueError, "invalid HTTP reason phrase: \\x7f"):
            await read_response(self.stream)

    async def test_read_response_invalid_header(self):
        self.stream.feed_data(b"HTTP/1.1 500 Internal Server Error\r\nOops\r\n")
        with self.assertRaisesRegex(ValueError, "invalid HTTP header line: Oops"):
            await read_response(self.stream)

    async def test_header_name(self):
        self.stream.feed_data(b"foo bar: baz qux\r\n\r\n")
        with self.assertRaises(ValueError):
            await read_headers(self.stream)

    async def test_header_value(self):
        self.stream.feed_data(b"foo: \x00\x00\x0f\r\n\r\n")
        with self.assertRaises(ValueError):
            await read_headers(self.stream)

    async def test_headers_limit(self):
        self.stream.feed_data(b"foo: bar\r\n" * 129 + b"\r\n")
        with self.assertRaises(SecurityError):
            await read_headers(self.stream)

    async def test_line_limit(self):
        # Header line contains 5 + 8186 + 2 = 8193 bytes.
        self.stream.feed_data(b"foo: " + b"a" * 8186 + b"\r\n\r\n")
        with self.assertRaises(SecurityError):
            await read_headers(self.stream)

    async def test_line_ending(self):
        self.stream.feed_data(b"foo: bar\n\n")
        with self.assertRaises(EOFError):
            await read_headers(self.stream)
