from websockets.datastructures import Headers
from websockets.exceptions import SecurityError
from websockets.http11 import *
from websockets.http11 import parse_headers
from websockets.streams import StreamReader

from .utils import GeneratorTestCase


class RequestTests(GeneratorTestCase):
    def setUp(self):
        super().setUp()
        self.reader = StreamReader()

    def parse(self):
        return Request.parse(self.reader.read_line)

    def test_parse(self):
        # Example from the protocol overview in RFC 6455
        self.reader.feed_data(
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
        request = self.assertGeneratorReturns(self.parse())
        self.assertEqual(request.path, "/chat")
        self.assertEqual(request.headers["Upgrade"], "websocket")

    def test_parse_empty(self):
        self.reader.feed_eof()
        with self.assertRaises(EOFError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "connection closed while reading HTTP request line",
        )

    def test_parse_invalid_request_line(self):
        self.reader.feed_data(b"GET /\r\n\r\n")
        with self.assertRaises(ValueError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "invalid HTTP request line: GET /",
        )

    def test_parse_unsupported_method(self):
        self.reader.feed_data(b"OPTIONS * HTTP/1.1\r\n\r\n")
        with self.assertRaises(ValueError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "unsupported HTTP method: OPTIONS",
        )

    def test_parse_unsupported_version(self):
        self.reader.feed_data(b"GET /chat HTTP/1.0\r\n\r\n")
        with self.assertRaises(ValueError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "unsupported HTTP version: HTTP/1.0",
        )

    def test_parse_invalid_header(self):
        self.reader.feed_data(b"GET /chat HTTP/1.1\r\nOops\r\n")
        with self.assertRaises(ValueError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "invalid HTTP header line: Oops",
        )

    def test_parse_body(self):
        self.reader.feed_data(b"GET / HTTP/1.1\r\nContent-Length: 3\r\n\r\nYo\n")
        with self.assertRaises(ValueError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "unsupported request body",
        )

    def test_parse_body_with_transfer_encoding(self):
        self.reader.feed_data(b"GET / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n")
        with self.assertRaises(NotImplementedError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "transfer codings aren't supported",
        )

    def test_serialize(self):
        # Example from the protocol overview in RFC 6455
        request = Request(
            "/chat",
            Headers(
                [
                    ("Host", "server.example.com"),
                    ("Upgrade", "websocket"),
                    ("Connection", "Upgrade"),
                    ("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ=="),
                    ("Origin", "http://example.com"),
                    ("Sec-WebSocket-Protocol", "chat, superchat"),
                    ("Sec-WebSocket-Version", "13"),
                ]
            ),
        )
        self.assertEqual(
            request.serialize(),
            b"GET /chat HTTP/1.1\r\n"
            b"Host: server.example.com\r\n"
            b"Upgrade: websocket\r\n"
            b"Connection: Upgrade\r\n"
            b"Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n"
            b"Origin: http://example.com\r\n"
            b"Sec-WebSocket-Protocol: chat, superchat\r\n"
            b"Sec-WebSocket-Version: 13\r\n"
            b"\r\n",
        )


class ResponseTests(GeneratorTestCase):
    def setUp(self):
        super().setUp()
        self.reader = StreamReader()

    def parse(self):
        return Response.parse(
            self.reader.read_line,
            self.reader.read_exact,
            self.reader.read_to_eof,
        )

    def test_parse(self):
        # Example from the protocol overview in RFC 6455
        self.reader.feed_data(
            b"HTTP/1.1 101 Switching Protocols\r\n"
            b"Upgrade: websocket\r\n"
            b"Connection: Upgrade\r\n"
            b"Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n"
            b"Sec-WebSocket-Protocol: chat\r\n"
            b"\r\n"
        )
        response = self.assertGeneratorReturns(self.parse())
        self.assertEqual(response.status_code, 101)
        self.assertEqual(response.reason_phrase, "Switching Protocols")
        self.assertEqual(response.headers["Upgrade"], "websocket")
        self.assertIsNone(response.body)

    def test_parse_empty(self):
        self.reader.feed_eof()
        with self.assertRaises(EOFError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "connection closed while reading HTTP status line",
        )

    def test_parse_invalid_status_line(self):
        self.reader.feed_data(b"Hello!\r\n")
        with self.assertRaises(ValueError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "invalid HTTP status line: Hello!",
        )

    def test_parse_unsupported_version(self):
        self.reader.feed_data(b"HTTP/1.0 400 Bad Request\r\n\r\n")
        with self.assertRaises(ValueError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "unsupported HTTP version: HTTP/1.0",
        )

    def test_parse_invalid_status(self):
        self.reader.feed_data(b"HTTP/1.1 OMG WTF\r\n\r\n")
        with self.assertRaises(ValueError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "invalid HTTP status code: OMG",
        )

    def test_parse_unsupported_status(self):
        self.reader.feed_data(b"HTTP/1.1 007 My name is Bond\r\n\r\n")
        with self.assertRaises(ValueError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "unsupported HTTP status code: 007",
        )

    def test_parse_invalid_reason(self):
        self.reader.feed_data(b"HTTP/1.1 200 \x7f\r\n\r\n")
        with self.assertRaises(ValueError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "invalid HTTP reason phrase: \x7f",
        )

    def test_parse_invalid_header(self):
        self.reader.feed_data(b"HTTP/1.1 500 Internal Server Error\r\nOops\r\n")
        with self.assertRaises(ValueError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "invalid HTTP header line: Oops",
        )

    def test_parse_body_with_content_length(self):
        self.reader.feed_data(
            b"HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello world!\n"
        )
        response = self.assertGeneratorReturns(self.parse())
        self.assertEqual(response.body, b"Hello world!\n")

    def test_parse_body_without_content_length(self):
        self.reader.feed_data(b"HTTP/1.1 200 OK\r\n\r\nHello world!\n")
        gen = self.parse()
        self.assertGeneratorRunning(gen)
        self.reader.feed_eof()
        response = self.assertGeneratorReturns(gen)
        self.assertEqual(response.body, b"Hello world!\n")

    def test_parse_body_with_content_length_too_long(self):
        self.reader.feed_data(b"HTTP/1.1 200 OK\r\nContent-Length: 1048577\r\n\r\n")
        with self.assertRaises(SecurityError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "body too large: 1048577 bytes",
        )

    def test_parse_body_without_content_length_too_long(self):
        self.reader.feed_data(b"HTTP/1.1 200 OK\r\n\r\n" + b"a" * 1048577)
        with self.assertRaises(SecurityError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "body too large: over 1048576 bytes",
        )

    def test_parse_body_with_transfer_encoding(self):
        self.reader.feed_data(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n")
        with self.assertRaises(NotImplementedError) as raised:
            next(self.parse())
        self.assertEqual(
            str(raised.exception),
            "transfer codings aren't supported",
        )

    def test_parse_body_no_content(self):
        self.reader.feed_data(b"HTTP/1.1 204 No Content\r\n\r\n")
        response = self.assertGeneratorReturns(self.parse())
        self.assertIsNone(response.body)

    def test_parse_body_not_modified(self):
        self.reader.feed_data(b"HTTP/1.1 304 Not Modified\r\n\r\n")
        response = self.assertGeneratorReturns(self.parse())
        self.assertIsNone(response.body)

    def test_serialize(self):
        # Example from the protocol overview in RFC 6455
        response = Response(
            101,
            "Switching Protocols",
            Headers(
                [
                    ("Upgrade", "websocket"),
                    ("Connection", "Upgrade"),
                    ("Sec-WebSocket-Accept", "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="),
                    ("Sec-WebSocket-Protocol", "chat"),
                ]
            ),
        )
        self.assertEqual(
            response.serialize(),
            b"HTTP/1.1 101 Switching Protocols\r\n"
            b"Upgrade: websocket\r\n"
            b"Connection: Upgrade\r\n"
            b"Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n"
            b"Sec-WebSocket-Protocol: chat\r\n"
            b"\r\n",
        )

    def test_serialize_with_body(self):
        response = Response(
            200,
            "OK",
            Headers([("Content-Length", "13"), ("Content-Type", "text/plain")]),
            b"Hello world!\n",
        )
        self.assertEqual(
            response.serialize(),
            b"HTTP/1.1 200 OK\r\n"
            b"Content-Length: 13\r\n"
            b"Content-Type: text/plain\r\n"
            b"\r\n"
            b"Hello world!\n",
        )


class HeadersTests(GeneratorTestCase):
    def setUp(self):
        super().setUp()
        self.reader = StreamReader()

    def parse_headers(self):
        return parse_headers(self.reader.read_line)

    def test_parse_invalid_name(self):
        self.reader.feed_data(b"foo bar: baz qux\r\n\r\n")
        with self.assertRaises(ValueError):
            next(self.parse_headers())

    def test_parse_invalid_value(self):
        self.reader.feed_data(b"foo: \x00\x00\x0f\r\n\r\n")
        with self.assertRaises(ValueError):
            next(self.parse_headers())

    def test_parse_too_long_value(self):
        self.reader.feed_data(b"foo: bar\r\n" * 129 + b"\r\n")
        with self.assertRaises(SecurityError):
            next(self.parse_headers())

    def test_parse_too_long_line(self):
        # Header line contains 5 + 8186 + 2 = 8193 bytes.
        self.reader.feed_data(b"foo: " + b"a" * 8186 + b"\r\n\r\n")
        with self.assertRaises(SecurityError):
            next(self.parse_headers())

    def test_parse_invalid_line_ending(self):
        self.reader.feed_data(b"foo: bar\n\n")
        with self.assertRaises(EOFError):
            next(self.parse_headers())
