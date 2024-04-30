import socket
import ssl
import threading
import unittest

from websockets.exceptions import InvalidHandshake
from websockets.extensions.permessage_deflate import PerMessageDeflate
from websockets.sync.client import *

from ..utils import MS, temp_unix_socket_path
from .client import CLIENT_CONTEXT, run_client, run_unix_client
from .server import SERVER_CONTEXT, do_nothing, run_server, run_unix_server


class ClientTests(unittest.TestCase):
    def test_connection(self):
        """Client connects to server and the handshake succeeds."""
        with run_server() as server:
            with run_client(server) as client:
                self.assertEqual(client.protocol.state.name, "OPEN")

    def test_connection_fails(self):
        """Client connects to server but the handshake fails."""

        def remove_accept_header(self, request, response):
            del response.headers["Sec-WebSocket-Accept"]

        # The connection will be open for the server but failed for the client.
        # Use a connection handler that exits immediately to avoid an exception.
        with run_server(do_nothing, process_response=remove_accept_header) as server:
            with self.assertRaisesRegex(
                InvalidHandshake,
                "missing Sec-WebSocket-Accept header",
            ):
                with run_client(server, close_timeout=MS):
                    self.fail("did not raise")

    def test_tcp_connection_fails(self):
        """Client fails to connect to server."""
        with self.assertRaises(OSError):
            with run_client("ws://localhost:54321"):  # invalid port
                self.fail("did not raise")

    def test_existing_socket(self):
        """Client connects using a pre-existing socket."""
        with run_server() as server:
            with socket.create_connection(server.socket.getsockname()) as sock:
                # Use a non-existing domain to ensure we connect to the right socket.
                with run_client("ws://invalid/", sock=sock) as client:
                    self.assertEqual(client.protocol.state.name, "OPEN")

    def test_additional_headers(self):
        """Client can set additional headers with additional_headers."""
        with run_server() as server:
            with run_client(
                server, additional_headers={"Authorization": "Bearer ..."}
            ) as client:
                self.assertEqual(client.request.headers["Authorization"], "Bearer ...")

    def test_override_user_agent(self):
        """Client can override User-Agent header with user_agent_header."""
        with run_server() as server:
            with run_client(server, user_agent_header="Smith") as client:
                self.assertEqual(client.request.headers["User-Agent"], "Smith")

    def test_remove_user_agent(self):
        """Client can remove User-Agent header with user_agent_header."""
        with run_server() as server:
            with run_client(server, user_agent_header=None) as client:
                self.assertNotIn("User-Agent", client.request.headers)

    def test_compression_is_enabled(self):
        """Client enables compression by default."""
        with run_server() as server:
            with run_client(server) as client:
                self.assertEqual(
                    [type(ext) for ext in client.protocol.extensions],
                    [PerMessageDeflate],
                )

    def test_disable_compression(self):
        """Client disables compression."""
        with run_server() as server:
            with run_client(server, compression=None) as client:
                self.assertEqual(client.protocol.extensions, [])

    def test_custom_connection_factory(self):
        """Client runs ClientConnection factory provided in create_connection."""

        def create_connection(*args, **kwargs):
            client = ClientConnection(*args, **kwargs)
            client.create_connection_ran = True
            return client

        with run_server() as server:
            with run_client(server, create_connection=create_connection) as client:
                self.assertTrue(client.create_connection_ran)

    def test_timeout_during_handshake(self):
        """Client times out before receiving handshake response from server."""
        gate = threading.Event()

        def stall_connection(self, request):
            gate.wait()

        # The connection will be open for the server but failed for the client.
        # Use a connection handler that exits immediately to avoid an exception.
        with run_server(do_nothing, process_request=stall_connection) as server:
            try:
                with self.assertRaisesRegex(
                    TimeoutError,
                    "timed out during handshake",
                ):
                    # While it shouldn't take 50ms to open a connection, this
                    # test becomes flaky in CI when setting a smaller timeout,
                    # even after increasing WEBSOCKETS_TESTS_TIMEOUT_FACTOR.
                    with run_client(server, open_timeout=5 * MS):
                        self.fail("did not raise")
            finally:
                gate.set()

    def test_connection_closed_during_handshake(self):
        """Client reads EOF before receiving handshake response from server."""

        def close_connection(self, request):
            self.close_socket()

        with run_server(process_request=close_connection) as server:
            with self.assertRaisesRegex(
                ConnectionError,
                "connection closed during handshake",
            ):
                with run_client(server):
                    self.fail("did not raise")


class SecureClientTests(unittest.TestCase):
    def test_connection(self):
        """Client connects to server securely."""
        with run_server(ssl_context=SERVER_CONTEXT) as server:
            with run_client(server, ssl_context=CLIENT_CONTEXT) as client:
                self.assertEqual(client.protocol.state.name, "OPEN")
                self.assertEqual(client.socket.version()[:3], "TLS")

    def test_set_server_hostname_implicitly(self):
        """Client sets server_hostname to the host in the WebSocket URI."""
        with temp_unix_socket_path() as path:
            with run_unix_server(path, ssl_context=SERVER_CONTEXT):
                with run_unix_client(
                    path,
                    ssl_context=CLIENT_CONTEXT,
                    uri="wss://overridden/",
                ) as client:
                    self.assertEqual(client.socket.server_hostname, "overridden")

    def test_set_server_hostname_explicitly(self):
        """Client sets server_hostname to the value provided in argument."""
        with temp_unix_socket_path() as path:
            with run_unix_server(path, ssl_context=SERVER_CONTEXT):
                with run_unix_client(
                    path,
                    ssl_context=CLIENT_CONTEXT,
                    server_hostname="overridden",
                ) as client:
                    self.assertEqual(client.socket.server_hostname, "overridden")

    def test_reject_invalid_server_certificate(self):
        """Client rejects certificate where server certificate isn't trusted."""
        with run_server(ssl_context=SERVER_CONTEXT) as server:
            with self.assertRaisesRegex(
                ssl.SSLCertVerificationError,
                r"certificate verify failed: self[ -]signed certificate",
            ):
                # The test certificate isn't trusted system-wide.
                with run_client(server, secure=True):
                    self.fail("did not raise")

    def test_reject_invalid_server_hostname(self):
        """Client rejects certificate where server hostname doesn't match."""
        with run_server(ssl_context=SERVER_CONTEXT) as server:
            with self.assertRaisesRegex(
                ssl.SSLCertVerificationError,
                r"certificate verify failed: Hostname mismatch",
            ):
                # This hostname isn't included in the test certificate.
                with run_client(
                    server, ssl_context=CLIENT_CONTEXT, server_hostname="invalid"
                ):
                    self.fail("did not raise")


@unittest.skipUnless(hasattr(socket, "AF_UNIX"), "this test requires Unix sockets")
class UnixClientTests(unittest.TestCase):
    def test_connection(self):
        """Client connects to server over a Unix socket."""
        with temp_unix_socket_path() as path:
            with run_unix_server(path):
                with run_unix_client(path) as client:
                    self.assertEqual(client.protocol.state.name, "OPEN")

    def test_set_host_header(self):
        """Client sets the Host header to the host in the WebSocket URI."""
        # This is part of the documented behavior of unix_connect().
        with temp_unix_socket_path() as path:
            with run_unix_server(path):
                with run_unix_client(path, uri="ws://overridden/") as client:
                    self.assertEqual(client.request.headers["Host"], "overridden")


@unittest.skipUnless(hasattr(socket, "AF_UNIX"), "this test requires Unix sockets")
class SecureUnixClientTests(unittest.TestCase):
    def test_connection(self):
        """Client connects to server securely over a Unix socket."""
        with temp_unix_socket_path() as path:
            with run_unix_server(path, ssl_context=SERVER_CONTEXT):
                with run_unix_client(path, ssl_context=CLIENT_CONTEXT) as client:
                    self.assertEqual(client.protocol.state.name, "OPEN")
                    self.assertEqual(client.socket.version()[:3], "TLS")

    def test_set_server_hostname(self):
        """Client sets server_hostname to the host in the WebSocket URI."""
        # This is part of the documented behavior of unix_connect().
        with temp_unix_socket_path() as path:
            with run_unix_server(path, ssl_context=SERVER_CONTEXT):
                with run_unix_client(
                    path,
                    ssl_context=CLIENT_CONTEXT,
                    uri="wss://overridden/",
                ) as client:
                    self.assertEqual(client.socket.server_hostname, "overridden")


class ClientUsageErrorsTests(unittest.TestCase):
    def test_ssl_context_without_secure_uri(self):
        """Client rejects ssl_context when URI isn't secure."""
        with self.assertRaisesRegex(
            TypeError,
            "ssl_context argument is incompatible with a ws:// URI",
        ):
            connect("ws://localhost/", ssl_context=CLIENT_CONTEXT)

    def test_unix_without_path_or_sock(self):
        """Unix client requires path when sock isn't provided."""
        with self.assertRaisesRegex(
            TypeError,
            "missing path argument",
        ):
            unix_connect()

    def test_unix_with_path_and_sock(self):
        """Unix client rejects path when sock is provided."""
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.addCleanup(sock.close)
        with self.assertRaisesRegex(
            TypeError,
            "path and sock arguments are incompatible",
        ):
            unix_connect(path="/", sock=sock)

    def test_invalid_subprotocol(self):
        """Client rejects single value of subprotocols."""
        with self.assertRaisesRegex(
            TypeError,
            "subprotocols must be a list",
        ):
            connect("ws://localhost/", subprotocols="chat")

    def test_unsupported_compression(self):
        """Client rejects incorrect value of compression."""
        with self.assertRaisesRegex(
            ValueError,
            "unsupported compression: False",
        ):
            connect("ws://localhost/", compression=False)
