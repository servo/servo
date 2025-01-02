import http
import logging
import unittest
import unittest.mock

from websockets.datastructures import Headers
from websockets.exceptions import (
    InvalidHeader,
    InvalidOrigin,
    InvalidUpgrade,
    NegotiationError,
)
from websockets.frames import OP_TEXT, Frame
from websockets.http11 import Request, Response
from websockets.protocol import CONNECTING, OPEN
from websockets.server import *

from .extensions.utils import (
    OpExtension,
    Rsv2Extension,
    ServerOpExtensionFactory,
    ServerRsv2ExtensionFactory,
)
from .test_utils import ACCEPT, KEY
from .utils import DATE, DeprecationTestCase


class ConnectTests(unittest.TestCase):
    def test_receive_connect(self):
        server = ServerProtocol()
        server.receive_data(
            (
                f"GET /test HTTP/1.1\r\n"
                f"Host: example.com\r\n"
                f"Upgrade: websocket\r\n"
                f"Connection: Upgrade\r\n"
                f"Sec-WebSocket-Key: {KEY}\r\n"
                f"Sec-WebSocket-Version: 13\r\n"
                f"\r\n"
            ).encode(),
        )
        [request] = server.events_received()
        self.assertIsInstance(request, Request)
        self.assertEqual(server.data_to_send(), [])
        self.assertFalse(server.close_expected())

    def test_connect_request(self):
        server = ServerProtocol()
        server.receive_data(
            (
                f"GET /test HTTP/1.1\r\n"
                f"Host: example.com\r\n"
                f"Upgrade: websocket\r\n"
                f"Connection: Upgrade\r\n"
                f"Sec-WebSocket-Key: {KEY}\r\n"
                f"Sec-WebSocket-Version: 13\r\n"
                f"\r\n"
            ).encode(),
        )
        [request] = server.events_received()
        self.assertEqual(request.path, "/test")
        self.assertEqual(
            request.headers,
            Headers(
                {
                    "Host": "example.com",
                    "Upgrade": "websocket",
                    "Connection": "Upgrade",
                    "Sec-WebSocket-Key": KEY,
                    "Sec-WebSocket-Version": "13",
                }
            ),
        )

    def test_no_request(self):
        server = ServerProtocol()
        server.receive_eof()
        self.assertEqual(server.events_received(), [])

    def test_partial_request(self):
        server = ServerProtocol()
        server.receive_data(b"GET /test HTTP/1.1\r\n")
        server.receive_eof()
        self.assertEqual(server.events_received(), [])

    def test_random_request(self):
        server = ServerProtocol()
        server.receive_data(b"HELO relay.invalid\r\n")
        server.receive_data(b"MAIL FROM: <alice@invalid>\r\n")
        server.receive_data(b"RCPT TO: <bob@invalid>\r\n")
        self.assertEqual(server.events_received(), [])


class AcceptRejectTests(unittest.TestCase):
    def make_request(self):
        return Request(
            path="/test",
            headers=Headers(
                {
                    "Host": "example.com",
                    "Upgrade": "websocket",
                    "Connection": "Upgrade",
                    "Sec-WebSocket-Key": KEY,
                    "Sec-WebSocket-Version": "13",
                }
            ),
        )

    def test_send_accept(self):
        server = ServerProtocol()
        with unittest.mock.patch("email.utils.formatdate", return_value=DATE):
            response = server.accept(self.make_request())
        self.assertIsInstance(response, Response)
        server.send_response(response)
        self.assertEqual(
            server.data_to_send(),
            [
                f"HTTP/1.1 101 Switching Protocols\r\n"
                f"Date: {DATE}\r\n"
                f"Upgrade: websocket\r\n"
                f"Connection: Upgrade\r\n"
                f"Sec-WebSocket-Accept: {ACCEPT}\r\n"
                f"\r\n".encode()
            ],
        )
        self.assertFalse(server.close_expected())
        self.assertEqual(server.state, OPEN)

    def test_send_reject(self):
        server = ServerProtocol()
        with unittest.mock.patch("email.utils.formatdate", return_value=DATE):
            response = server.reject(http.HTTPStatus.NOT_FOUND, "Sorry folks.\n")
        self.assertIsInstance(response, Response)
        server.send_response(response)
        self.assertEqual(
            server.data_to_send(),
            [
                f"HTTP/1.1 404 Not Found\r\n"
                f"Date: {DATE}\r\n"
                f"Connection: close\r\n"
                f"Content-Length: 13\r\n"
                f"Content-Type: text/plain; charset=utf-8\r\n"
                f"\r\n"
                f"Sorry folks.\n".encode(),
                b"",
            ],
        )
        self.assertTrue(server.close_expected())
        self.assertEqual(server.state, CONNECTING)

    def test_accept_response(self):
        server = ServerProtocol()
        with unittest.mock.patch("email.utils.formatdate", return_value=DATE):
            response = server.accept(self.make_request())
        self.assertIsInstance(response, Response)
        self.assertEqual(response.status_code, 101)
        self.assertEqual(response.reason_phrase, "Switching Protocols")
        self.assertEqual(
            response.headers,
            Headers(
                {
                    "Date": DATE,
                    "Upgrade": "websocket",
                    "Connection": "Upgrade",
                    "Sec-WebSocket-Accept": ACCEPT,
                }
            ),
        )
        self.assertIsNone(response.body)

    def test_reject_response(self):
        server = ServerProtocol()
        with unittest.mock.patch("email.utils.formatdate", return_value=DATE):
            response = server.reject(http.HTTPStatus.NOT_FOUND, "Sorry folks.\n")
        self.assertIsInstance(response, Response)
        self.assertEqual(response.status_code, 404)
        self.assertEqual(response.reason_phrase, "Not Found")
        self.assertEqual(
            response.headers,
            Headers(
                {
                    "Date": DATE,
                    "Connection": "close",
                    "Content-Length": "13",
                    "Content-Type": "text/plain; charset=utf-8",
                }
            ),
        )
        self.assertEqual(response.body, b"Sorry folks.\n")

    def test_reject_response_supports_int_status(self):
        server = ServerProtocol()
        response = server.reject(404, "Sorry folks.\n")
        self.assertEqual(response.status_code, 404)
        self.assertEqual(response.reason_phrase, "Not Found")

    def test_basic(self):
        server = ServerProtocol()
        request = self.make_request()
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)

    def test_unexpected_exception(self):
        server = ServerProtocol()
        request = self.make_request()
        with unittest.mock.patch(
            "websockets.server.ServerProtocol.process_request",
            side_effect=Exception("BOOM"),
        ):
            response = server.accept(request)

        self.assertEqual(response.status_code, 500)
        with self.assertRaises(Exception) as raised:
            raise server.handshake_exc
        self.assertEqual(str(raised.exception), "BOOM")

    def test_missing_connection(self):
        server = ServerProtocol()
        request = self.make_request()
        del request.headers["Connection"]
        response = server.accept(request)

        self.assertEqual(response.status_code, 426)
        self.assertEqual(response.headers["Upgrade"], "websocket")
        with self.assertRaises(InvalidUpgrade) as raised:
            raise server.handshake_exc
        self.assertEqual(str(raised.exception), "missing Connection header")

    def test_invalid_connection(self):
        server = ServerProtocol()
        request = self.make_request()
        del request.headers["Connection"]
        request.headers["Connection"] = "close"
        response = server.accept(request)

        self.assertEqual(response.status_code, 426)
        self.assertEqual(response.headers["Upgrade"], "websocket")
        with self.assertRaises(InvalidUpgrade) as raised:
            raise server.handshake_exc
        self.assertEqual(str(raised.exception), "invalid Connection header: close")

    def test_missing_upgrade(self):
        server = ServerProtocol()
        request = self.make_request()
        del request.headers["Upgrade"]
        response = server.accept(request)

        self.assertEqual(response.status_code, 426)
        self.assertEqual(response.headers["Upgrade"], "websocket")
        with self.assertRaises(InvalidUpgrade) as raised:
            raise server.handshake_exc
        self.assertEqual(str(raised.exception), "missing Upgrade header")

    def test_invalid_upgrade(self):
        server = ServerProtocol()
        request = self.make_request()
        del request.headers["Upgrade"]
        request.headers["Upgrade"] = "h2c"
        response = server.accept(request)

        self.assertEqual(response.status_code, 426)
        self.assertEqual(response.headers["Upgrade"], "websocket")
        with self.assertRaises(InvalidUpgrade) as raised:
            raise server.handshake_exc
        self.assertEqual(str(raised.exception), "invalid Upgrade header: h2c")

    def test_missing_key(self):
        server = ServerProtocol()
        request = self.make_request()
        del request.headers["Sec-WebSocket-Key"]
        response = server.accept(request)

        self.assertEqual(response.status_code, 400)
        with self.assertRaises(InvalidHeader) as raised:
            raise server.handshake_exc
        self.assertEqual(str(raised.exception), "missing Sec-WebSocket-Key header")

    def test_multiple_key(self):
        server = ServerProtocol()
        request = self.make_request()
        request.headers["Sec-WebSocket-Key"] = KEY
        response = server.accept(request)

        self.assertEqual(response.status_code, 400)
        with self.assertRaises(InvalidHeader) as raised:
            raise server.handshake_exc
        self.assertEqual(
            str(raised.exception),
            "invalid Sec-WebSocket-Key header: "
            "more than one Sec-WebSocket-Key header found",
        )

    def test_invalid_key(self):
        server = ServerProtocol()
        request = self.make_request()
        del request.headers["Sec-WebSocket-Key"]
        request.headers["Sec-WebSocket-Key"] = "not Base64 data!"
        response = server.accept(request)

        self.assertEqual(response.status_code, 400)
        with self.assertRaises(InvalidHeader) as raised:
            raise server.handshake_exc
        self.assertEqual(
            str(raised.exception), "invalid Sec-WebSocket-Key header: not Base64 data!"
        )

    def test_truncated_key(self):
        server = ServerProtocol()
        request = self.make_request()
        del request.headers["Sec-WebSocket-Key"]
        request.headers["Sec-WebSocket-Key"] = KEY[
            :16
        ]  # 12 bytes instead of 16, Base64-encoded
        response = server.accept(request)

        self.assertEqual(response.status_code, 400)
        with self.assertRaises(InvalidHeader) as raised:
            raise server.handshake_exc
        self.assertEqual(
            str(raised.exception), f"invalid Sec-WebSocket-Key header: {KEY[:16]}"
        )

    def test_missing_version(self):
        server = ServerProtocol()
        request = self.make_request()
        del request.headers["Sec-WebSocket-Version"]
        response = server.accept(request)

        self.assertEqual(response.status_code, 400)
        with self.assertRaises(InvalidHeader) as raised:
            raise server.handshake_exc
        self.assertEqual(str(raised.exception), "missing Sec-WebSocket-Version header")

    def test_multiple_version(self):
        server = ServerProtocol()
        request = self.make_request()
        request.headers["Sec-WebSocket-Version"] = "11"
        response = server.accept(request)

        self.assertEqual(response.status_code, 400)
        with self.assertRaises(InvalidHeader) as raised:
            raise server.handshake_exc
        self.assertEqual(
            str(raised.exception),
            "invalid Sec-WebSocket-Version header: "
            "more than one Sec-WebSocket-Version header found",
        )

    def test_invalid_version(self):
        server = ServerProtocol()
        request = self.make_request()
        del request.headers["Sec-WebSocket-Version"]
        request.headers["Sec-WebSocket-Version"] = "11"
        response = server.accept(request)

        self.assertEqual(response.status_code, 400)
        with self.assertRaises(InvalidHeader) as raised:
            raise server.handshake_exc
        self.assertEqual(
            str(raised.exception), "invalid Sec-WebSocket-Version header: 11"
        )

    def test_no_origin(self):
        server = ServerProtocol(origins=["https://example.com"])
        request = self.make_request()
        response = server.accept(request)

        self.assertEqual(response.status_code, 403)
        with self.assertRaises(InvalidOrigin) as raised:
            raise server.handshake_exc
        self.assertEqual(str(raised.exception), "missing Origin header")

    def test_origin(self):
        server = ServerProtocol(origins=["https://example.com"])
        request = self.make_request()
        request.headers["Origin"] = "https://example.com"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertEqual(server.origin, "https://example.com")

    def test_unexpected_origin(self):
        server = ServerProtocol(origins=["https://example.com"])
        request = self.make_request()
        request.headers["Origin"] = "https://other.example.com"
        response = server.accept(request)

        self.assertEqual(response.status_code, 403)
        with self.assertRaises(InvalidOrigin) as raised:
            raise server.handshake_exc
        self.assertEqual(
            str(raised.exception), "invalid Origin header: https://other.example.com"
        )

    def test_multiple_origin(self):
        server = ServerProtocol(
            origins=["https://example.com", "https://other.example.com"]
        )
        request = self.make_request()
        request.headers["Origin"] = "https://example.com"
        request.headers["Origin"] = "https://other.example.com"
        response = server.accept(request)

        # This is prohibited by the HTTP specification, so the return code is
        # 400 Bad Request rather than 403 Forbidden.
        self.assertEqual(response.status_code, 400)
        with self.assertRaises(InvalidHeader) as raised:
            raise server.handshake_exc
        self.assertEqual(
            str(raised.exception),
            "invalid Origin header: more than one Origin header found",
        )

    def test_supported_origin(self):
        server = ServerProtocol(
            origins=["https://example.com", "https://other.example.com"]
        )
        request = self.make_request()
        request.headers["Origin"] = "https://other.example.com"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertEqual(server.origin, "https://other.example.com")

    def test_unsupported_origin(self):
        server = ServerProtocol(
            origins=["https://example.com", "https://other.example.com"]
        )
        request = self.make_request()
        request.headers["Origin"] = "https://original.example.com"
        response = server.accept(request)

        self.assertEqual(response.status_code, 403)
        with self.assertRaises(InvalidOrigin) as raised:
            raise server.handshake_exc
        self.assertEqual(
            str(raised.exception), "invalid Origin header: https://original.example.com"
        )

    def test_no_origin_accepted(self):
        server = ServerProtocol(origins=[None])
        request = self.make_request()
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertIsNone(server.origin)

    def test_no_extensions(self):
        server = ServerProtocol()
        request = self.make_request()
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertNotIn("Sec-WebSocket-Extensions", response.headers)
        self.assertEqual(server.extensions, [])

    def test_no_extension(self):
        server = ServerProtocol(extensions=[ServerOpExtensionFactory()])
        request = self.make_request()
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertNotIn("Sec-WebSocket-Extensions", response.headers)
        self.assertEqual(server.extensions, [])

    def test_extension(self):
        server = ServerProtocol(extensions=[ServerOpExtensionFactory()])
        request = self.make_request()
        request.headers["Sec-WebSocket-Extensions"] = "x-op; op"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertEqual(response.headers["Sec-WebSocket-Extensions"], "x-op; op")
        self.assertEqual(server.extensions, [OpExtension()])

    def test_unexpected_extension(self):
        server = ServerProtocol()
        request = self.make_request()
        request.headers["Sec-WebSocket-Extensions"] = "x-op; op"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertNotIn("Sec-WebSocket-Extensions", response.headers)
        self.assertEqual(server.extensions, [])

    def test_unsupported_extension(self):
        server = ServerProtocol(extensions=[ServerRsv2ExtensionFactory()])
        request = self.make_request()
        request.headers["Sec-WebSocket-Extensions"] = "x-op; op"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertNotIn("Sec-WebSocket-Extensions", response.headers)
        self.assertEqual(server.extensions, [])

    def test_supported_extension_parameters(self):
        server = ServerProtocol(extensions=[ServerOpExtensionFactory("this")])
        request = self.make_request()
        request.headers["Sec-WebSocket-Extensions"] = "x-op; op=this"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertEqual(response.headers["Sec-WebSocket-Extensions"], "x-op; op=this")
        self.assertEqual(server.extensions, [OpExtension("this")])

    def test_unsupported_extension_parameters(self):
        server = ServerProtocol(extensions=[ServerOpExtensionFactory("this")])
        request = self.make_request()
        request.headers["Sec-WebSocket-Extensions"] = "x-op; op=that"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertNotIn("Sec-WebSocket-Extensions", response.headers)
        self.assertEqual(server.extensions, [])

    def test_multiple_supported_extension_parameters(self):
        server = ServerProtocol(
            extensions=[
                ServerOpExtensionFactory("this"),
                ServerOpExtensionFactory("that"),
            ]
        )
        request = self.make_request()
        request.headers["Sec-WebSocket-Extensions"] = "x-op; op=that"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertEqual(response.headers["Sec-WebSocket-Extensions"], "x-op; op=that")
        self.assertEqual(server.extensions, [OpExtension("that")])

    def test_multiple_extensions(self):
        server = ServerProtocol(
            extensions=[ServerOpExtensionFactory(), ServerRsv2ExtensionFactory()]
        )
        request = self.make_request()
        request.headers["Sec-WebSocket-Extensions"] = "x-op; op"
        request.headers["Sec-WebSocket-Extensions"] = "x-rsv2"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertEqual(
            response.headers["Sec-WebSocket-Extensions"], "x-op; op, x-rsv2"
        )
        self.assertEqual(server.extensions, [OpExtension(), Rsv2Extension()])

    def test_multiple_extensions_order(self):
        server = ServerProtocol(
            extensions=[ServerOpExtensionFactory(), ServerRsv2ExtensionFactory()]
        )
        request = self.make_request()
        request.headers["Sec-WebSocket-Extensions"] = "x-rsv2"
        request.headers["Sec-WebSocket-Extensions"] = "x-op; op"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertEqual(
            response.headers["Sec-WebSocket-Extensions"], "x-rsv2, x-op; op"
        )
        self.assertEqual(server.extensions, [Rsv2Extension(), OpExtension()])

    def test_no_subprotocols(self):
        server = ServerProtocol()
        request = self.make_request()
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertNotIn("Sec-WebSocket-Protocol", response.headers)
        self.assertIsNone(server.subprotocol)

    def test_no_subprotocol(self):
        server = ServerProtocol(subprotocols=["chat"])
        request = self.make_request()
        response = server.accept(request)

        self.assertEqual(response.status_code, 400)
        with self.assertRaisesRegex(
            NegotiationError,
            r"missing subprotocol",
        ):
            raise server.handshake_exc

    def test_subprotocol(self):
        server = ServerProtocol(subprotocols=["chat"])
        request = self.make_request()
        request.headers["Sec-WebSocket-Protocol"] = "chat"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertEqual(response.headers["Sec-WebSocket-Protocol"], "chat")
        self.assertEqual(server.subprotocol, "chat")

    def test_unexpected_subprotocol(self):
        server = ServerProtocol()
        request = self.make_request()
        request.headers["Sec-WebSocket-Protocol"] = "chat"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertNotIn("Sec-WebSocket-Protocol", response.headers)
        self.assertIsNone(server.subprotocol)

    def test_multiple_subprotocols(self):
        server = ServerProtocol(subprotocols=["superchat", "chat"])
        request = self.make_request()
        request.headers["Sec-WebSocket-Protocol"] = "chat"
        request.headers["Sec-WebSocket-Protocol"] = "superchat"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertEqual(response.headers["Sec-WebSocket-Protocol"], "superchat")
        self.assertEqual(server.subprotocol, "superchat")

    def test_supported_subprotocol(self):
        server = ServerProtocol(subprotocols=["superchat", "chat"])
        request = self.make_request()
        request.headers["Sec-WebSocket-Protocol"] = "chat"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertEqual(response.headers["Sec-WebSocket-Protocol"], "chat")
        self.assertEqual(server.subprotocol, "chat")

    def test_unsupported_subprotocol(self):
        server = ServerProtocol(subprotocols=["superchat", "chat"])
        request = self.make_request()
        request.headers["Sec-WebSocket-Protocol"] = "otherchat"
        response = server.accept(request)

        self.assertEqual(response.status_code, 400)
        with self.assertRaisesRegex(
            NegotiationError,
            r"invalid subprotocol; expected one of superchat, chat",
        ):
            raise server.handshake_exc

    @staticmethod
    def optional_chat(protocol, subprotocols):
        if "chat" in subprotocols:
            return "chat"

    def test_select_subprotocol(self):
        server = ServerProtocol(select_subprotocol=self.optional_chat)
        request = self.make_request()
        request.headers["Sec-WebSocket-Protocol"] = "chat"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertEqual(response.headers["Sec-WebSocket-Protocol"], "chat")
        self.assertEqual(server.subprotocol, "chat")

    def test_select_no_subprotocol(self):
        server = ServerProtocol(select_subprotocol=self.optional_chat)
        request = self.make_request()
        request.headers["Sec-WebSocket-Protocol"] = "otherchat"
        response = server.accept(request)

        self.assertEqual(response.status_code, 101)
        self.assertNotIn("Sec-WebSocket-Protocol", response.headers)
        self.assertIsNone(server.subprotocol)


class MiscTests(unittest.TestCase):
    def test_bypass_handshake(self):
        server = ServerProtocol(state=OPEN)
        server.receive_data(b"\x81\x86\x00\x00\x00\x00Hello!")
        [frame] = server.events_received()
        self.assertEqual(frame, Frame(OP_TEXT, b"Hello!"))

    def test_custom_logger(self):
        logger = logging.getLogger("test")
        with self.assertLogs("test", logging.DEBUG) as logs:
            ServerProtocol(logger=logger)
        self.assertEqual(len(logs.records), 1)


class BackwardsCompatibilityTests(DeprecationTestCase):
    def test_server_connection_class(self):
        with self.assertDeprecationWarning(
            "ServerConnection was renamed to ServerProtocol"
        ):
            from websockets.server import ServerConnection

            server = ServerConnection()

        self.assertIsInstance(server, ServerProtocol)
