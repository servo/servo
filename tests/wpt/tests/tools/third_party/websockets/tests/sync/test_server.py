import dataclasses
import http
import logging
import socket
import threading
import unittest

from websockets.exceptions import (
    ConnectionClosedError,
    ConnectionClosedOK,
    InvalidStatus,
    NegotiationError,
)
from websockets.http11 import Request, Response
from websockets.sync.server import *

from ..utils import MS, temp_unix_socket_path
from .client import CLIENT_CONTEXT, run_client, run_unix_client
from .server import (
    SERVER_CONTEXT,
    EvalShellMixin,
    crash,
    do_nothing,
    eval_shell,
    run_server,
    run_unix_server,
)


class ServerTests(EvalShellMixin, unittest.TestCase):
    def test_connection(self):
        """Server receives connection from client and the handshake succeeds."""
        with run_server() as server:
            with run_client(server) as client:
                self.assertEval(client, "ws.protocol.state.name", "OPEN")

    def test_connection_fails(self):
        """Server receives connection from client but the handshake fails."""

        def remove_key_header(self, request):
            del request.headers["Sec-WebSocket-Key"]

        with run_server(process_request=remove_key_header) as server:
            with self.assertRaisesRegex(
                InvalidStatus,
                "server rejected WebSocket connection: HTTP 400",
            ):
                with run_client(server):
                    self.fail("did not raise")

    def test_connection_handler_returns(self):
        """Connection handler returns."""
        with run_server(do_nothing) as server:
            with run_client(server) as client:
                with self.assertRaisesRegex(
                    ConnectionClosedOK,
                    r"received 1000 \(OK\); then sent 1000 \(OK\)",
                ):
                    client.recv()

    def test_connection_handler_raises_exception(self):
        """Connection handler raises an exception."""
        with run_server(crash) as server:
            with run_client(server) as client:
                with self.assertRaisesRegex(
                    ConnectionClosedError,
                    r"received 1011 \(internal error\); "
                    r"then sent 1011 \(internal error\)",
                ):
                    client.recv()

    def test_existing_socket(self):
        """Server receives connection using a pre-existing socket."""
        with socket.create_server(("localhost", 0)) as sock:
            with run_server(sock=sock):
                # Build WebSocket URI to ensure we connect to the right socket.
                with run_client("ws://{}:{}/".format(*sock.getsockname())) as client:
                    self.assertEval(client, "ws.protocol.state.name", "OPEN")

    def test_select_subprotocol(self):
        """Server selects a subprotocol with the select_subprotocol callable."""

        def select_subprotocol(ws, subprotocols):
            ws.select_subprotocol_ran = True
            assert "chat" in subprotocols
            return "chat"

        with run_server(
            subprotocols=["chat"],
            select_subprotocol=select_subprotocol,
        ) as server:
            with run_client(server, subprotocols=["chat"]) as client:
                self.assertEval(client, "ws.select_subprotocol_ran", "True")
                self.assertEval(client, "ws.subprotocol", "chat")

    def test_select_subprotocol_rejects_handshake(self):
        """Server rejects handshake if select_subprotocol raises NegotiationError."""

        def select_subprotocol(ws, subprotocols):
            raise NegotiationError

        with run_server(select_subprotocol=select_subprotocol) as server:
            with self.assertRaisesRegex(
                InvalidStatus,
                "server rejected WebSocket connection: HTTP 400",
            ):
                with run_client(server):
                    self.fail("did not raise")

    def test_select_subprotocol_raises_exception(self):
        """Server returns an error if select_subprotocol raises an exception."""

        def select_subprotocol(ws, subprotocols):
            raise RuntimeError

        with run_server(select_subprotocol=select_subprotocol) as server:
            with self.assertRaisesRegex(
                InvalidStatus,
                "server rejected WebSocket connection: HTTP 500",
            ):
                with run_client(server):
                    self.fail("did not raise")

    def test_process_request(self):
        """Server runs process_request before processing the handshake."""

        def process_request(ws, request):
            self.assertIsInstance(request, Request)
            ws.process_request_ran = True

        with run_server(process_request=process_request) as server:
            with run_client(server) as client:
                self.assertEval(client, "ws.process_request_ran", "True")

    def test_process_request_abort_handshake(self):
        """Server aborts handshake if process_request returns a response."""

        def process_request(ws, request):
            return ws.protocol.reject(http.HTTPStatus.FORBIDDEN, "Forbidden")

        with run_server(process_request=process_request) as server:
            with self.assertRaisesRegex(
                InvalidStatus,
                "server rejected WebSocket connection: HTTP 403",
            ):
                with run_client(server):
                    self.fail("did not raise")

    def test_process_request_raises_exception(self):
        """Server returns an error if process_request raises an exception."""

        def process_request(ws, request):
            raise RuntimeError

        with run_server(process_request=process_request) as server:
            with self.assertRaisesRegex(
                InvalidStatus,
                "server rejected WebSocket connection: HTTP 500",
            ):
                with run_client(server):
                    self.fail("did not raise")

    def test_process_response(self):
        """Server runs process_response after processing the handshake."""

        def process_response(ws, request, response):
            self.assertIsInstance(request, Request)
            self.assertIsInstance(response, Response)
            ws.process_response_ran = True

        with run_server(process_response=process_response) as server:
            with run_client(server) as client:
                self.assertEval(client, "ws.process_response_ran", "True")

    def test_process_response_override_response(self):
        """Server runs process_response after processing the handshake."""

        def process_response(ws, request, response):
            headers = response.headers.copy()
            headers["X-ProcessResponse-Ran"] = "true"
            return dataclasses.replace(response, headers=headers)

        with run_server(process_response=process_response) as server:
            with run_client(server) as client:
                self.assertEqual(
                    client.response.headers["X-ProcessResponse-Ran"], "true"
                )

    def test_process_response_raises_exception(self):
        """Server returns an error if process_response raises an exception."""

        def process_response(ws, request, response):
            raise RuntimeError

        with run_server(process_response=process_response) as server:
            with self.assertRaisesRegex(
                InvalidStatus,
                "server rejected WebSocket connection: HTTP 500",
            ):
                with run_client(server):
                    self.fail("did not raise")

    def test_override_server(self):
        """Server can override Server header with server_header."""
        with run_server(server_header="Neo") as server:
            with run_client(server) as client:
                self.assertEval(client, "ws.response.headers['Server']", "Neo")

    def test_remove_server(self):
        """Server can remove Server header with server_header."""
        with run_server(server_header=None) as server:
            with run_client(server) as client:
                self.assertEval(client, "'Server' in ws.response.headers", "False")

    def test_compression_is_enabled(self):
        """Server enables compression by default."""
        with run_server() as server:
            with run_client(server) as client:
                self.assertEval(
                    client,
                    "[type(ext).__name__ for ext in ws.protocol.extensions]",
                    "['PerMessageDeflate']",
                )

    def test_disable_compression(self):
        """Server disables compression."""
        with run_server(compression=None) as server:
            with run_client(server) as client:
                self.assertEval(client, "ws.protocol.extensions", "[]")

    def test_custom_connection_factory(self):
        """Server runs ServerConnection factory provided in create_connection."""

        def create_connection(*args, **kwargs):
            server = ServerConnection(*args, **kwargs)
            server.create_connection_ran = True
            return server

        with run_server(create_connection=create_connection) as server:
            with run_client(server) as client:
                self.assertEval(client, "ws.create_connection_ran", "True")

    def test_timeout_during_handshake(self):
        """Server times out before receiving handshake request from client."""
        with run_server(open_timeout=MS) as server:
            with socket.create_connection(server.socket.getsockname()) as sock:
                self.assertEqual(sock.recv(4096), b"")

    def test_connection_closed_during_handshake(self):
        """Server reads EOF before receiving handshake request from client."""
        with run_server() as server:
            # Patch handler to record a reference to the thread running it.
            server_thread = None
            conn_received = threading.Event()
            original_handler = server.handler

            def handler(sock, addr):
                nonlocal server_thread
                server_thread = threading.current_thread()
                nonlocal conn_received
                conn_received.set()
                original_handler(sock, addr)

            server.handler = handler

            with socket.create_connection(server.socket.getsockname()):
                # Wait for the server to receive the connection, then close it.
                conn_received.wait()

            # Wait for the server thread to terminate.
            server_thread.join()


class SecureServerTests(EvalShellMixin, unittest.TestCase):
    def test_connection(self):
        """Server receives secure connection from client."""
        with run_server(ssl_context=SERVER_CONTEXT) as server:
            with run_client(server, ssl_context=CLIENT_CONTEXT) as client:
                self.assertEval(client, "ws.protocol.state.name", "OPEN")
                self.assertEval(client, "ws.socket.version()[:3]", "TLS")

    def test_timeout_during_tls_handshake(self):
        """Server times out before receiving TLS handshake request from client."""
        with run_server(ssl_context=SERVER_CONTEXT, open_timeout=MS) as server:
            with socket.create_connection(server.socket.getsockname()) as sock:
                self.assertEqual(sock.recv(4096), b"")

    def test_connection_closed_during_tls_handshake(self):
        """Server reads EOF before receiving TLS handshake request from client."""
        with run_server(ssl_context=SERVER_CONTEXT) as server:
            # Patch handler to record a reference to the thread running it.
            server_thread = None
            conn_received = threading.Event()
            original_handler = server.handler

            def handler(sock, addr):
                nonlocal server_thread
                server_thread = threading.current_thread()
                nonlocal conn_received
                conn_received.set()
                original_handler(sock, addr)

            server.handler = handler

            with socket.create_connection(server.socket.getsockname()):
                # Wait for the server to receive the connection, then close it.
                conn_received.wait()

            # Wait for the server thread to terminate.
            server_thread.join()


@unittest.skipUnless(hasattr(socket, "AF_UNIX"), "this test requires Unix sockets")
class UnixServerTests(EvalShellMixin, unittest.TestCase):
    def test_connection(self):
        """Server receives connection from client over a Unix socket."""
        with temp_unix_socket_path() as path:
            with run_unix_server(path):
                with run_unix_client(path) as client:
                    self.assertEval(client, "ws.protocol.state.name", "OPEN")


@unittest.skipUnless(hasattr(socket, "AF_UNIX"), "this test requires Unix sockets")
class SecureUnixServerTests(EvalShellMixin, unittest.TestCase):
    def test_connection(self):
        """Server receives secure connection from client over a Unix socket."""
        with temp_unix_socket_path() as path:
            with run_unix_server(path, ssl_context=SERVER_CONTEXT):
                with run_unix_client(path, ssl_context=CLIENT_CONTEXT) as client:
                    self.assertEval(client, "ws.protocol.state.name", "OPEN")
                    self.assertEval(client, "ws.socket.version()[:3]", "TLS")


class ServerUsageErrorsTests(unittest.TestCase):
    def test_unix_without_path_or_sock(self):
        """Unix server requires path when sock isn't provided."""
        with self.assertRaisesRegex(
            TypeError,
            "missing path argument",
        ):
            unix_serve(eval_shell)

    def test_unix_with_path_and_sock(self):
        """Unix server rejects path when sock is provided."""
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.addCleanup(sock.close)
        with self.assertRaisesRegex(
            TypeError,
            "path and sock arguments are incompatible",
        ):
            unix_serve(eval_shell, path="/", sock=sock)

    def test_invalid_subprotocol(self):
        """Server rejects single value of subprotocols."""
        with self.assertRaisesRegex(
            TypeError,
            "subprotocols must be a list",
        ):
            serve(eval_shell, subprotocols="chat")

    def test_unsupported_compression(self):
        """Server rejects incorrect value of compression."""
        with self.assertRaisesRegex(
            ValueError,
            "unsupported compression: False",
        ):
            serve(eval_shell, compression=False)


class WebSocketServerTests(unittest.TestCase):
    def test_logger(self):
        """WebSocketServer accepts a logger argument."""
        logger = logging.getLogger("test")
        with run_server(logger=logger) as server:
            self.assertIs(server.logger, logger)

    def test_fileno(self):
        """WebSocketServer provides a fileno attribute."""
        with run_server() as server:
            self.assertIsInstance(server.fileno(), int)

    def test_shutdown(self):
        """WebSocketServer provides a shutdown method."""
        with run_server() as server:
            server.shutdown()
            # Check that the server socket is closed.
            with self.assertRaises(OSError):
                server.socket.accept()
