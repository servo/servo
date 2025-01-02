import unittest

from websockets.datastructures import Headers
from websockets.exceptions import *
from websockets.frames import Close, CloseCode
from websockets.http11 import Response


class ExceptionsTests(unittest.TestCase):
    def test_str(self):
        for exception, exception_str in [
            (
                WebSocketException("something went wrong"),
                "something went wrong",
            ),
            (
                ConnectionClosed(
                    Close(CloseCode.NORMAL_CLOSURE, ""),
                    Close(CloseCode.NORMAL_CLOSURE, ""),
                    True,
                ),
                "received 1000 (OK); then sent 1000 (OK)",
            ),
            (
                ConnectionClosed(
                    Close(CloseCode.GOING_AWAY, "Bye!"),
                    Close(CloseCode.GOING_AWAY, "Bye!"),
                    False,
                ),
                "sent 1001 (going away) Bye!; then received 1001 (going away) Bye!",
            ),
            (
                ConnectionClosed(
                    Close(CloseCode.NORMAL_CLOSURE, "race"),
                    Close(CloseCode.NORMAL_CLOSURE, "cond"),
                    True,
                ),
                "received 1000 (OK) race; then sent 1000 (OK) cond",
            ),
            (
                ConnectionClosed(
                    Close(CloseCode.NORMAL_CLOSURE, "cond"),
                    Close(CloseCode.NORMAL_CLOSURE, "race"),
                    False,
                ),
                "sent 1000 (OK) race; then received 1000 (OK) cond",
            ),
            (
                ConnectionClosed(
                    None,
                    Close(CloseCode.MESSAGE_TOO_BIG, ""),
                    None,
                ),
                "sent 1009 (message too big); no close frame received",
            ),
            (
                ConnectionClosed(
                    Close(CloseCode.PROTOCOL_ERROR, ""),
                    None,
                    None,
                ),
                "received 1002 (protocol error); no close frame sent",
            ),
            (
                ConnectionClosedOK(
                    Close(CloseCode.NORMAL_CLOSURE, ""),
                    Close(CloseCode.NORMAL_CLOSURE, ""),
                    True,
                ),
                "received 1000 (OK); then sent 1000 (OK)",
            ),
            (
                ConnectionClosedError(
                    None,
                    None,
                    None,
                ),
                "no close frame received or sent",
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
                InvalidHeaderFormat("Sec-WebSocket-Protocol", "exp. token", "a=|", 3),
                "invalid Sec-WebSocket-Protocol header: exp. token at 3 in a=|",
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
                InvalidStatus(Response(401, "Unauthorized", Headers())),
                "server rejected WebSocket connection: HTTP 401",
            ),
            (
                InvalidStatusCode(403, Headers()),
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
                InvalidURI("|", "not at all!"),
                "| isn't a valid URI: not at all!",
            ),
            (
                PayloadTooBig("payload length exceeds limit: 2 > 1 bytes"),
                "payload length exceeds limit: 2 > 1 bytes",
            ),
            (
                ProtocolError("invalid opcode: 7"),
                "invalid opcode: 7",
            ),
        ]:
            with self.subTest(exception=exception):
                self.assertEqual(str(exception), exception_str)

    def test_connection_closed_attributes_backwards_compatibility(self):
        exception = ConnectionClosed(Close(CloseCode.NORMAL_CLOSURE, "OK"), None, None)
        self.assertEqual(exception.code, CloseCode.NORMAL_CLOSURE)
        self.assertEqual(exception.reason, "OK")

    def test_connection_closed_attributes_backwards_compatibility_defaults(self):
        exception = ConnectionClosed(None, None, None)
        self.assertEqual(exception.code, CloseCode.ABNORMAL_CLOSURE)
        self.assertEqual(exception.reason, "")
