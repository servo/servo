import unittest

from websockets.exceptions import *
from websockets.http import Headers


class ExceptionsTests(unittest.TestCase):
    def test_str(self):
        for exception, exception_str in [
            # fmt: off
            (
                WebSocketException("something went wrong"),
                "something went wrong",
            ),
            (
                ConnectionClosed(1000, ""),
                "code = 1000 (OK), no reason",
            ),
            (
                ConnectionClosed(1006, None),
                "code = 1006 (connection closed abnormally [internal]), no reason"
            ),
            (
                ConnectionClosed(3000, None),
                "code = 3000 (registered), no reason"
            ),
            (
                ConnectionClosed(4000, None),
                "code = 4000 (private use), no reason"
            ),
            (
                ConnectionClosedError(1016, None),
                "code = 1016 (unknown), no reason"
            ),
            (
                ConnectionClosedOK(1001, "bye"),
                "code = 1001 (going away), reason = bye",
            ),
            (
                InvalidHandshake("invalid request"),
                "invalid request",
            ),
            (
                SecurityError("redirect from WSS to WS"),
                "redirect from WSS to WS",
            ),
            (
                InvalidMessage("malformed HTTP message"),
                "malformed HTTP message",
            ),
            (
                InvalidHeader("Name"),
                "missing Name header",
            ),
            (
                InvalidHeader("Name", None),
                "missing Name header",
            ),
            (
                InvalidHeader("Name", ""),
                "empty Name header",
            ),
            (
                InvalidHeader("Name", "Value"),
                "invalid Name header: Value",
            ),
            (
                InvalidHeaderFormat(
                    "Sec-WebSocket-Protocol", "expected token", "a=|", 3
                ),
                "invalid Sec-WebSocket-Protocol header: "
                "expected token at 3 in a=|",
            ),
            (
                InvalidHeaderValue("Sec-WebSocket-Version", "42"),
                "invalid Sec-WebSocket-Version header: 42",
            ),
            (
                InvalidOrigin("http://bad.origin"),
                "invalid Origin header: http://bad.origin",
            ),
            (
                InvalidUpgrade("Upgrade"),
                "missing Upgrade header",
            ),
            (
                InvalidUpgrade("Connection", "websocket"),
                "invalid Connection header: websocket",
            ),
            (
                InvalidStatusCode(403),
                "server rejected WebSocket connection: HTTP 403",
            ),
            (
                NegotiationError("unsupported subprotocol: spam"),
                "unsupported subprotocol: spam",
            ),
            (
                DuplicateParameter("a"),
                "duplicate parameter: a",
            ),
            (
                InvalidParameterName("|"),
                "invalid parameter name: |",
            ),
            (
                InvalidParameterValue("a", None),
                "missing value for parameter a",
            ),
            (
                InvalidParameterValue("a", ""),
                "empty value for parameter a",
            ),
            (
                InvalidParameterValue("a", "|"),
                "invalid value for parameter a: |",
            ),
            (
                AbortHandshake(200, Headers(), b"OK\n"),
                "HTTP 200, 0 headers, 3 bytes",
            ),
            (
                RedirectHandshake("wss://example.com"),
                "redirect to wss://example.com",
            ),
            (
                InvalidState("WebSocket connection isn't established yet"),
                "WebSocket connection isn't established yet",
            ),
            (
                InvalidURI("|"),
                "| isn't a valid URI",
            ),
            (
                PayloadTooBig("payload length exceeds limit: 2 > 1 bytes"),
                "payload length exceeds limit: 2 > 1 bytes",
            ),
            (
                ProtocolError("invalid opcode: 7"),
                "invalid opcode: 7",
            ),
            # fmt: on
        ]:
            with self.subTest(exception=exception):
                self.assertEqual(str(exception), exception_str)
