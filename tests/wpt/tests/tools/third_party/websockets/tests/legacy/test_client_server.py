import asyncio
import contextlib
import functools
import http
import logging
import platform
import random
import socket
import ssl
import sys
import unittest
import unittest.mock
import urllib.error
import urllib.request
import warnings

from websockets.datastructures import Headers
from websockets.exceptions import (
    ConnectionClosed,
    InvalidHandshake,
    InvalidHeader,
    InvalidStatusCode,
    NegotiationError,
)
from websockets.extensions.permessage_deflate import (
    ClientPerMessageDeflateFactory,
    PerMessageDeflate,
    ServerPerMessageDeflateFactory,
)
from websockets.frames import CloseCode
from websockets.http import USER_AGENT
from websockets.legacy.client import *
from websockets.legacy.compatibility import asyncio_timeout
from websockets.legacy.handshake import build_response
from websockets.legacy.http import read_response
from websockets.legacy.server import *
from websockets.protocol import State
from websockets.uri import parse_uri

from ..extensions.utils import (
    ClientNoOpExtensionFactory,
    NoOpExtension,
    ServerNoOpExtensionFactory,
)
from ..utils import CERTIFICATE, MS, temp_unix_socket_path
from .utils import AsyncioTestCase


async def default_handler(ws):
    if ws.path == "/deprecated_attributes":
        await ws.recv()  # delay that allows catching warnings
        await ws.send(repr((ws.host, ws.port, ws.secure)))
    elif ws.path == "/close_timeout":
        await ws.send(repr(ws.close_timeout))
    elif ws.path == "/path":
        await ws.send(str(ws.path))
    elif ws.path == "/headers":
        await ws.send(repr(ws.request_headers))
        await ws.send(repr(ws.response_headers))
    elif ws.path == "/extensions":
        await ws.send(repr(ws.extensions))
    elif ws.path == "/subprotocol":
        await ws.send(repr(ws.subprotocol))
    elif ws.path == "/slow_stop":
        await ws.wait_closed()
        await asyncio.sleep(2 * MS)
    else:
        await ws.send((await ws.recv()))


async def redirect_request(path, headers, test, status):
    if path == "/absolute_redirect":
        location = get_server_uri(test.server, test.secure, "/")
    elif path == "/relative_redirect":
        location = "/"
    elif path == "/infinite":
        location = get_server_uri(test.server, test.secure, "/infinite")
    elif path == "/force_insecure":
        location = get_server_uri(test.server, False, "/")
    elif path == "/missing_location":
        return status, {}, b""
    else:
        return None
    return status, {"Location": location}, b""


@contextlib.contextmanager
def temp_test_server(test, **kwargs):
    test.start_server(**kwargs)
    try:
        yield
    finally:
        test.stop_server()


def temp_test_redirecting_server(test, status=http.HTTPStatus.FOUND, **kwargs):
    process_request = functools.partial(redirect_request, test=test, status=status)
    return temp_test_server(test, process_request=process_request, **kwargs)


@contextlib.contextmanager
def temp_test_client(test, *args, **kwargs):
    test.start_client(*args, **kwargs)
    try:
        yield
    finally:
        test.stop_client()


def with_manager(manager, *args, **kwargs):
    """
    Return a decorator that wraps a function with a context manager.

    """

    def decorate(func):
        @functools.wraps(func)
        def _decorate(self, *_args, **_kwargs):
            with manager(self, *args, **kwargs):
                return func(self, *_args, **_kwargs)

        return _decorate

    return decorate


def with_server(**kwargs):
    """
    Return a decorator for TestCase methods that starts and stops a server.

    """
    return with_manager(temp_test_server, **kwargs)


def with_client(*args, **kwargs):
    """
    Return a decorator for TestCase methods that starts and stops a client.

    """
    return with_manager(temp_test_client, *args, **kwargs)


def get_server_address(server):
    """
    Return an address on which the given server listens.

    """
    # Pick a random socket in order to test both IPv4 and IPv6 on systems
    # where both are available. Randomizing tests is usually a bad idea. If
    # needed, either use the first socket, or test separately IPv4 and IPv6.
    server_socket = random.choice(server.sockets)

    if server_socket.family == socket.AF_INET6:  # pragma: no cover
        return server_socket.getsockname()[:2]  # (no IPv6 on CI)
    elif server_socket.family == socket.AF_INET:
        return server_socket.getsockname()
    else:  # pragma: no cover
        raise ValueError("expected an IPv6, IPv4, or Unix socket")


def get_server_uri(server, secure=False, resource_name="/", user_info=None):
    """
    Return a WebSocket URI for connecting to the given server.

    """
    proto = "wss" if secure else "ws"
    user_info = ":".join(user_info) + "@" if user_info else ""
    host, port = get_server_address(server)
    if ":" in host:  # IPv6 address
        host = f"[{host}]"
    return f"{proto}://{user_info}{host}:{port}{resource_name}"


class UnauthorizedServerProtocol(WebSocketServerProtocol):
    async def process_request(self, path, request_headers):
        # Test returning headers as a Headers instance (1/3)
        return http.HTTPStatus.UNAUTHORIZED, Headers([("X-Access", "denied")]), b""


class ForbiddenServerProtocol(WebSocketServerProtocol):
    async def process_request(self, path, request_headers):
        # Test returning headers as a dict (2/3)
        return http.HTTPStatus.FORBIDDEN, {"X-Access": "denied"}, b""


class HealthCheckServerProtocol(WebSocketServerProtocol):
    async def process_request(self, path, request_headers):
        # Test returning headers as a list of pairs (3/3)
        if path == "/__health__/":
            return http.HTTPStatus.OK, [("X-Access", "OK")], b"status = green\n"


class ProcessRequestReturningIntProtocol(WebSocketServerProtocol):
    async def process_request(self, path, request_headers):
        assert path == "/__health__/"
        return 200, [], b"OK\n"


class SlowOpeningHandshakeProtocol(WebSocketServerProtocol):
    async def process_request(self, path, request_headers):
        await asyncio.sleep(10 * MS)


class FooClientProtocol(WebSocketClientProtocol):
    pass


class BarClientProtocol(WebSocketClientProtocol):
    pass


class ClientServerTestsMixin:
    secure = False

    def setUp(self):
        super().setUp()
        self.server = None

    def start_server(self, deprecation_warnings=None, **kwargs):
        handler = kwargs.pop("handler", default_handler)
        # Disable compression by default in tests.
        kwargs.setdefault("compression", None)
        # Disable pings by default in tests.
        kwargs.setdefault("ping_interval", None)

        # This logic is encapsulated in a coroutine to prevent it from executing
        # before the event loop is running which causes asyncio.get_event_loop()
        # to raise a DeprecationWarning on Python ≥ 3.10.
        async def start_server():
            return await serve(handler, "localhost", 0, **kwargs)

        with warnings.catch_warnings(record=True) as recorded_warnings:
            warnings.simplefilter("always")
            self.server = self.loop.run_until_complete(start_server())

        expected_warnings = [] if deprecation_warnings is None else deprecation_warnings
        self.assertDeprecationWarnings(recorded_warnings, expected_warnings)

    def start_client(
        self, resource_name="/", user_info=None, deprecation_warnings=None, **kwargs
    ):
        # Disable compression by default in tests.
        kwargs.setdefault("compression", None)
        # Disable pings by default in tests.
        kwargs.setdefault("ping_interval", None)

        secure = kwargs.get("ssl") is not None
        try:
            server_uri = kwargs.pop("uri")
        except KeyError:
            server_uri = get_server_uri(self.server, secure, resource_name, user_info)

        # This logic is encapsulated in a coroutine to prevent it from executing
        # before the event loop is running which causes asyncio.get_event_loop()
        # to raise a DeprecationWarning on Python ≥ 3.10.
        async def start_client():
            return await connect(server_uri, **kwargs)

        with warnings.catch_warnings(record=True) as recorded_warnings:
            warnings.simplefilter("always")
            self.client = self.loop.run_until_complete(start_client())

        expected_warnings = [] if deprecation_warnings is None else deprecation_warnings
        self.assertDeprecationWarnings(recorded_warnings, expected_warnings)

    def stop_client(self):
        self.loop.run_until_complete(
            asyncio.wait_for(self.client.close_connection_task, timeout=1)
        )

    def stop_server(self):
        self.server.close()
        self.loop.run_until_complete(
            asyncio.wait_for(self.server.wait_closed(), timeout=1)
        )

    @contextlib.contextmanager
    def temp_server(self, **kwargs):
        with temp_test_server(self, **kwargs):
            yield

    @contextlib.contextmanager
    def temp_client(self, *args, **kwargs):
        with temp_test_client(self, *args, **kwargs):
            yield

    def make_http_request(self, path="/", headers=None):
        if headers is None:
            headers = {}

        # Set url to 'https?://<host>:<port><path>'.
        url = get_server_uri(
            self.server, resource_name=path, secure=self.secure
        ).replace("ws", "http")

        request = urllib.request.Request(url=url, headers=headers)

        if self.secure:
            open_health_check = functools.partial(
                urllib.request.urlopen, request, context=self.client_context
            )
        else:
            open_health_check = functools.partial(urllib.request.urlopen, request)

        return self.loop.run_in_executor(None, open_health_check)


class SecureClientServerTestsMixin(ClientServerTestsMixin):
    secure = True

    @property
    def server_context(self):
        ssl_context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        ssl_context.load_cert_chain(CERTIFICATE)
        return ssl_context

    @property
    def client_context(self):
        ssl_context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
        ssl_context.load_verify_locations(CERTIFICATE)
        return ssl_context

    def start_server(self, **kwargs):
        kwargs.setdefault("ssl", self.server_context)
        super().start_server(**kwargs)

    def start_client(self, path="/", **kwargs):
        kwargs.setdefault("ssl", self.client_context)
        super().start_client(path, **kwargs)


class CommonClientServerTests:
    """
    Mixin that defines most tests but doesn't inherit unittest.TestCase.

    Tests are run by the ClientServerTests and SecureClientServerTests subclasses.

    """

    @with_server()
    @with_client()
    def test_basic(self):
        self.loop.run_until_complete(self.client.send("Hello!"))
        reply = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(reply, "Hello!")

    def test_redirect(self):
        redirect_statuses = [
            http.HTTPStatus.MOVED_PERMANENTLY,
            http.HTTPStatus.FOUND,
            http.HTTPStatus.SEE_OTHER,
            http.HTTPStatus.TEMPORARY_REDIRECT,
            http.HTTPStatus.PERMANENT_REDIRECT,
        ]
        for status in redirect_statuses:
            with temp_test_redirecting_server(self, status):
                with self.temp_client("/absolute_redirect"):
                    self.loop.run_until_complete(self.client.send("Hello!"))
                    reply = self.loop.run_until_complete(self.client.recv())
                    self.assertEqual(reply, "Hello!")

    def test_redirect_relative_location(self):
        with temp_test_redirecting_server(self):
            with self.temp_client("/relative_redirect"):
                self.loop.run_until_complete(self.client.send("Hello!"))
                reply = self.loop.run_until_complete(self.client.recv())
                self.assertEqual(reply, "Hello!")

    def test_infinite_redirect(self):
        with temp_test_redirecting_server(self):
            with self.assertRaises(InvalidHandshake):
                with self.temp_client("/infinite"):
                    self.fail("did not raise")

    def test_redirect_missing_location(self):
        with temp_test_redirecting_server(self):
            with self.assertRaises(InvalidHeader):
                with self.temp_client("/missing_location"):
                    self.fail("did not raise")

    def test_loop_backwards_compatibility(self):
        with self.temp_server(
            loop=self.loop,
            deprecation_warnings=["remove loop argument"],
        ):
            with self.temp_client(
                loop=self.loop,
                deprecation_warnings=["remove loop argument"],
            ):
                self.loop.run_until_complete(self.client.send("Hello!"))
                reply = self.loop.run_until_complete(self.client.recv())
                self.assertEqual(reply, "Hello!")

    @with_server()
    def test_explicit_host_port(self):
        uri = get_server_uri(self.server, self.secure)
        wsuri = parse_uri(uri)

        # Change host and port to invalid values.
        scheme = "wss" if wsuri.secure else "ws"
        port = 65535 - wsuri.port
        changed_uri = f"{scheme}://example.com:{port}/"

        with self.temp_client(uri=changed_uri, host=wsuri.host, port=wsuri.port):
            self.loop.run_until_complete(self.client.send("Hello!"))
            reply = self.loop.run_until_complete(self.client.recv())
            self.assertEqual(reply, "Hello!")

    @with_server()
    def test_explicit_socket(self):
        class TrackedSocket(socket.socket):
            def __init__(self, *args, **kwargs):
                self.used_for_read = False
                self.used_for_write = False
                super().__init__(*args, **kwargs)

            def recv(self, *args, **kwargs):
                self.used_for_read = True
                return super().recv(*args, **kwargs)

            def recv_into(self, *args, **kwargs):
                self.used_for_read = True
                return super().recv_into(*args, **kwargs)

            def send(self, *args, **kwargs):
                self.used_for_write = True
                return super().send(*args, **kwargs)

        server_socket = [
            sock for sock in self.server.sockets if sock.family == socket.AF_INET
        ][0]
        client_socket = TrackedSocket(socket.AF_INET, socket.SOCK_STREAM)
        client_socket.connect(server_socket.getsockname())

        try:
            self.assertFalse(client_socket.used_for_read)
            self.assertFalse(client_socket.used_for_write)

            with self.temp_client(sock=client_socket):
                self.loop.run_until_complete(self.client.send("Hello!"))
                reply = self.loop.run_until_complete(self.client.recv())
                self.assertEqual(reply, "Hello!")

            self.assertTrue(client_socket.used_for_read)
            self.assertTrue(client_socket.used_for_write)

        finally:
            client_socket.close()

    @unittest.skipUnless(hasattr(socket, "AF_UNIX"), "this test requires Unix sockets")
    def test_unix_socket(self):
        with temp_unix_socket_path() as path:
            # Like self.start_server() but with unix_serve().
            async def start_server():
                return await unix_serve(default_handler, path)

            self.server = self.loop.run_until_complete(start_server())

            try:
                # Like self.start_client() but with unix_connect()
                async def start_client():
                    return await unix_connect(path)

                self.client = self.loop.run_until_complete(start_client())

                try:
                    self.loop.run_until_complete(self.client.send("Hello!"))
                    reply = self.loop.run_until_complete(self.client.recv())
                    self.assertEqual(reply, "Hello!")

                finally:
                    self.stop_client()

            finally:
                self.stop_server()

    def test_ws_handler_argument_backwards_compatibility(self):
        async def handler_with_path(ws, path):
            await ws.send(path)

        with self.temp_server(
            handler=handler_with_path,
            # Enable deprecation warning and announce deprecation in 11.0.
            # deprecation_warnings=["remove second argument of ws_handler"],
        ):
            with self.temp_client("/path"):
                self.assertEqual(
                    self.loop.run_until_complete(self.client.recv()),
                    "/path",
                )

    def test_ws_handler_argument_backwards_compatibility_partial(self):
        async def handler_with_path(ws, path, extra):
            await ws.send(path)

        bound_handler_with_path = functools.partial(handler_with_path, extra=None)

        with self.temp_server(
            handler=bound_handler_with_path,
            # Enable deprecation warning and announce deprecation in 11.0.
            # deprecation_warnings=["remove second argument of ws_handler"],
        ):
            with self.temp_client("/path"):
                self.assertEqual(
                    self.loop.run_until_complete(self.client.recv()),
                    "/path",
                )

    async def process_request_OK(path, request_headers):
        return http.HTTPStatus.OK, [], b"OK\n"

    @with_server(process_request=process_request_OK)
    def test_process_request_argument(self):
        response = self.loop.run_until_complete(self.make_http_request("/"))

        with contextlib.closing(response):
            self.assertEqual(response.code, 200)

    def legacy_process_request_OK(path, request_headers):
        return http.HTTPStatus.OK, [], b"OK\n"

    @with_server(process_request=legacy_process_request_OK)
    def test_process_request_argument_backwards_compatibility(self):
        with warnings.catch_warnings(record=True) as recorded_warnings:
            warnings.simplefilter("always")
            response = self.loop.run_until_complete(self.make_http_request("/"))

        with contextlib.closing(response):
            self.assertEqual(response.code, 200)

        self.assertDeprecationWarnings(
            recorded_warnings, ["declare process_request as a coroutine"]
        )

    class ProcessRequestOKServerProtocol(WebSocketServerProtocol):
        async def process_request(self, path, request_headers):
            return http.HTTPStatus.OK, [], b"OK\n"

    @with_server(create_protocol=ProcessRequestOKServerProtocol)
    def test_process_request_override(self):
        response = self.loop.run_until_complete(self.make_http_request("/"))

        with contextlib.closing(response):
            self.assertEqual(response.code, 200)

    class LegacyProcessRequestOKServerProtocol(WebSocketServerProtocol):
        def process_request(self, path, request_headers):
            return http.HTTPStatus.OK, [], b"OK\n"

    @with_server(create_protocol=LegacyProcessRequestOKServerProtocol)
    def test_process_request_override_backwards_compatibility(self):
        with warnings.catch_warnings(record=True) as recorded_warnings:
            warnings.simplefilter("always")
            response = self.loop.run_until_complete(self.make_http_request("/"))

        with contextlib.closing(response):
            self.assertEqual(response.code, 200)

        self.assertDeprecationWarnings(
            recorded_warnings, ["declare process_request as a coroutine"]
        )

    def select_subprotocol_chat(client_subprotocols, server_subprotocols):
        return "chat"

    @with_server(
        subprotocols=["superchat", "chat"], select_subprotocol=select_subprotocol_chat
    )
    @with_client("/subprotocol", subprotocols=["superchat", "chat"])
    def test_select_subprotocol_argument(self):
        server_subprotocol = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(server_subprotocol, repr("chat"))
        self.assertEqual(self.client.subprotocol, "chat")

    class SelectSubprotocolChatServerProtocol(WebSocketServerProtocol):
        def select_subprotocol(self, client_subprotocols, server_subprotocols):
            return "chat"

    @with_server(
        subprotocols=["superchat", "chat"],
        create_protocol=SelectSubprotocolChatServerProtocol,
    )
    @with_client("/subprotocol", subprotocols=["superchat", "chat"])
    def test_select_subprotocol_override(self):
        server_subprotocol = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(server_subprotocol, repr("chat"))
        self.assertEqual(self.client.subprotocol, "chat")

    @with_server()
    @with_client("/deprecated_attributes")
    def test_protocol_deprecated_attributes(self):
        # The test could be connecting with IPv6 or IPv4.
        expected_client_attrs = [
            server_socket.getsockname()[:2] + (self.secure,)
            for server_socket in self.server.sockets
        ]
        with warnings.catch_warnings(record=True) as recorded_warnings:
            warnings.simplefilter("always")
            client_attrs = (self.client.host, self.client.port, self.client.secure)
        self.assertDeprecationWarnings(
            recorded_warnings,
            [
                "use remote_address[0] instead of host",
                "use remote_address[1] instead of port",
                "don't use secure",
            ],
        )
        self.assertIn(client_attrs, expected_client_attrs)

        expected_server_attrs = ("localhost", 0, self.secure)
        with warnings.catch_warnings(record=True) as recorded_warnings:
            warnings.simplefilter("always")
            self.loop.run_until_complete(self.client.send(""))
            server_attrs = self.loop.run_until_complete(self.client.recv())
        self.assertDeprecationWarnings(
            recorded_warnings,
            [
                "use local_address[0] instead of host",
                "use local_address[1] instead of port",
                "don't use secure",
            ],
        )
        self.assertEqual(server_attrs, repr(expected_server_attrs))

    @with_server()
    @with_client("/path")
    def test_protocol_path(self):
        client_path = self.client.path
        self.assertEqual(client_path, "/path")
        server_path = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(server_path, "/path")

    @with_server()
    @with_client("/headers")
    def test_protocol_headers(self):
        client_req = self.client.request_headers
        client_resp = self.client.response_headers
        self.assertEqual(client_req["User-Agent"], USER_AGENT)
        self.assertEqual(client_resp["Server"], USER_AGENT)
        server_req = self.loop.run_until_complete(self.client.recv())
        server_resp = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(server_req, repr(client_req))
        self.assertEqual(server_resp, repr(client_resp))

    @with_server()
    @with_client("/headers", extra_headers={"X-Spam": "Eggs"})
    def test_protocol_custom_request_headers(self):
        req_headers = self.loop.run_until_complete(self.client.recv())
        self.loop.run_until_complete(self.client.recv())
        self.assertIn("('X-Spam', 'Eggs')", req_headers)

    @with_server()
    @with_client("/headers", extra_headers={"User-Agent": "websockets"})
    def test_protocol_custom_user_agent_header_legacy(self):
        req_headers = self.loop.run_until_complete(self.client.recv())
        self.loop.run_until_complete(self.client.recv())
        self.assertEqual(req_headers.count("User-Agent"), 1)
        self.assertIn("('User-Agent', 'websockets')", req_headers)

    @with_server()
    @with_client("/headers", user_agent_header=None)
    def test_protocol_no_user_agent_header(self):
        req_headers = self.loop.run_until_complete(self.client.recv())
        self.loop.run_until_complete(self.client.recv())
        self.assertNotIn("User-Agent", req_headers)

    @with_server()
    @with_client("/headers", user_agent_header="websockets")
    def test_protocol_custom_user_agent_header(self):
        req_headers = self.loop.run_until_complete(self.client.recv())
        self.loop.run_until_complete(self.client.recv())
        self.assertEqual(req_headers.count("User-Agent"), 1)
        self.assertIn("('User-Agent', 'websockets')", req_headers)

    @with_server(extra_headers=lambda p, r: {"X-Spam": "Eggs"})
    @with_client("/headers")
    def test_protocol_custom_response_headers_callable(self):
        self.loop.run_until_complete(self.client.recv())
        resp_headers = self.loop.run_until_complete(self.client.recv())
        self.assertIn("('X-Spam', 'Eggs')", resp_headers)

    @with_server(extra_headers=lambda p, r: None)
    @with_client("/headers")
    def test_protocol_custom_response_headers_callable_none(self):
        self.loop.run_until_complete(self.client.recv())  # doesn't crash
        self.loop.run_until_complete(self.client.recv())  # nothing to check

    @with_server(extra_headers={"X-Spam": "Eggs"})
    @with_client("/headers")
    def test_protocol_custom_response_headers(self):
        self.loop.run_until_complete(self.client.recv())
        resp_headers = self.loop.run_until_complete(self.client.recv())
        self.assertIn("('X-Spam', 'Eggs')", resp_headers)

    @with_server(extra_headers={"Server": "websockets"})
    @with_client("/headers")
    def test_protocol_custom_server_header_legacy(self):
        self.loop.run_until_complete(self.client.recv())
        resp_headers = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(resp_headers.count("Server"), 1)
        self.assertIn("('Server', 'websockets')", resp_headers)

    @with_server(server_header=None)
    @with_client("/headers")
    def test_protocol_no_server_header(self):
        self.loop.run_until_complete(self.client.recv())
        resp_headers = self.loop.run_until_complete(self.client.recv())
        self.assertNotIn("Server", resp_headers)

    @with_server(server_header="websockets")
    @with_client("/headers")
    def test_protocol_custom_server_header(self):
        self.loop.run_until_complete(self.client.recv())
        resp_headers = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(resp_headers.count("Server"), 1)
        self.assertIn("('Server', 'websockets')", resp_headers)

    @with_server(create_protocol=HealthCheckServerProtocol)
    def test_http_request_http_endpoint(self):
        # Making an HTTP request to an HTTP endpoint succeeds.
        response = self.loop.run_until_complete(self.make_http_request("/__health__/"))

        with contextlib.closing(response):
            self.assertEqual(response.code, 200)
            self.assertEqual(response.read(), b"status = green\n")

    @with_server(create_protocol=HealthCheckServerProtocol)
    def test_http_request_ws_endpoint(self):
        # Making an HTTP request to a WS endpoint fails.
        with self.assertRaises(urllib.error.HTTPError) as raised:
            self.loop.run_until_complete(self.make_http_request())

        self.assertEqual(raised.exception.code, 426)
        self.assertEqual(raised.exception.headers["Upgrade"], "websocket")

    @with_server(create_protocol=HealthCheckServerProtocol)
    def test_ws_connection_http_endpoint(self):
        # Making a WS connection to an HTTP endpoint fails.
        with self.assertRaises(InvalidStatusCode) as raised:
            self.start_client("/__health__/")

        self.assertEqual(raised.exception.status_code, 200)

    @with_server(create_protocol=HealthCheckServerProtocol)
    def test_ws_connection_ws_endpoint(self):
        # Making a WS connection to a WS endpoint succeeds.
        self.start_client()
        self.loop.run_until_complete(self.client.send("Hello!"))
        self.loop.run_until_complete(self.client.recv())
        self.stop_client()

    @with_server(create_protocol=HealthCheckServerProtocol, server_header=None)
    def test_http_request_no_server_header(self):
        response = self.loop.run_until_complete(self.make_http_request("/__health__/"))

        with contextlib.closing(response):
            self.assertNotIn("Server", response.headers)

    @with_server(create_protocol=HealthCheckServerProtocol, server_header="websockets")
    def test_http_request_custom_server_header(self):
        response = self.loop.run_until_complete(self.make_http_request("/__health__/"))

        with contextlib.closing(response):
            self.assertEqual(response.headers["Server"], "websockets")

    @with_server(create_protocol=ProcessRequestReturningIntProtocol)
    def test_process_request_returns_int_status(self):
        response = self.loop.run_until_complete(self.make_http_request("/__health__/"))

        with contextlib.closing(response):
            self.assertEqual(response.code, 200)
            self.assertEqual(response.read(), b"OK\n")

    def assert_client_raises_code(self, status_code):
        with self.assertRaises(InvalidStatusCode) as raised:
            self.start_client()
        self.assertEqual(raised.exception.status_code, status_code)

    @with_server(create_protocol=UnauthorizedServerProtocol)
    def test_server_create_protocol(self):
        self.assert_client_raises_code(401)

    def create_unauthorized_server_protocol(*args, **kwargs):
        return UnauthorizedServerProtocol(*args, **kwargs)

    @with_server(create_protocol=create_unauthorized_server_protocol)
    def test_server_create_protocol_function(self):
        self.assert_client_raises_code(401)

    @with_server(
        klass=UnauthorizedServerProtocol,
        deprecation_warnings=["rename klass to create_protocol"],
    )
    def test_server_klass_backwards_compatibility(self):
        self.assert_client_raises_code(401)

    @with_server(
        create_protocol=ForbiddenServerProtocol,
        klass=UnauthorizedServerProtocol,
        deprecation_warnings=["rename klass to create_protocol"],
    )
    def test_server_create_protocol_over_klass(self):
        self.assert_client_raises_code(403)

    @with_server()
    @with_client("/path", create_protocol=FooClientProtocol)
    def test_client_create_protocol(self):
        self.assertIsInstance(self.client, FooClientProtocol)

    @with_server()
    @with_client(
        "/path",
        create_protocol=(lambda *args, **kwargs: FooClientProtocol(*args, **kwargs)),
    )
    def test_client_create_protocol_function(self):
        self.assertIsInstance(self.client, FooClientProtocol)

    @with_server()
    @with_client(
        "/path",
        klass=FooClientProtocol,
        deprecation_warnings=["rename klass to create_protocol"],
    )
    def test_client_klass(self):
        self.assertIsInstance(self.client, FooClientProtocol)

    @with_server()
    @with_client(
        "/path",
        create_protocol=BarClientProtocol,
        klass=FooClientProtocol,
        deprecation_warnings=["rename klass to create_protocol"],
    )
    def test_client_create_protocol_over_klass(self):
        self.assertIsInstance(self.client, BarClientProtocol)

    @with_server(close_timeout=7)
    @with_client("/close_timeout")
    def test_server_close_timeout(self):
        close_timeout = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(eval(close_timeout), 7)

    @with_server(timeout=6, deprecation_warnings=["rename timeout to close_timeout"])
    @with_client("/close_timeout")
    def test_server_timeout_backwards_compatibility(self):
        close_timeout = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(eval(close_timeout), 6)

    @with_server(
        close_timeout=7,
        timeout=6,
        deprecation_warnings=["rename timeout to close_timeout"],
    )
    @with_client("/close_timeout")
    def test_server_close_timeout_over_timeout(self):
        close_timeout = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(eval(close_timeout), 7)

    @with_server()
    @with_client("/close_timeout", close_timeout=7)
    def test_client_close_timeout(self):
        self.assertEqual(self.client.close_timeout, 7)

    @with_server()
    @with_client(
        "/close_timeout",
        timeout=6,
        deprecation_warnings=["rename timeout to close_timeout"],
    )
    def test_client_timeout_backwards_compatibility(self):
        self.assertEqual(self.client.close_timeout, 6)

    @with_server()
    @with_client(
        "/close_timeout",
        close_timeout=7,
        timeout=6,
        deprecation_warnings=["rename timeout to close_timeout"],
    )
    def test_client_close_timeout_over_timeout(self):
        self.assertEqual(self.client.close_timeout, 7)

    @with_server()
    @with_client("/extensions")
    def test_no_extension(self):
        server_extensions = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(server_extensions, repr([]))
        self.assertEqual(repr(self.client.extensions), repr([]))

    @with_server(extensions=[ServerNoOpExtensionFactory()])
    @with_client("/extensions", extensions=[ClientNoOpExtensionFactory()])
    def test_extension(self):
        server_extensions = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(server_extensions, repr([NoOpExtension()]))
        self.assertEqual(repr(self.client.extensions), repr([NoOpExtension()]))

    @with_server()
    @with_client("/extensions", extensions=[ClientNoOpExtensionFactory()])
    def test_extension_not_accepted(self):
        server_extensions = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(server_extensions, repr([]))
        self.assertEqual(repr(self.client.extensions), repr([]))

    @with_server(extensions=[ServerNoOpExtensionFactory()])
    @with_client("/extensions")
    def test_extension_not_requested(self):
        server_extensions = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(server_extensions, repr([]))
        self.assertEqual(repr(self.client.extensions), repr([]))

    @with_server(extensions=[ServerNoOpExtensionFactory([("foo", None)])])
    def test_extension_client_rejection(self):
        with self.assertRaises(NegotiationError):
            self.start_client("/extensions", extensions=[ClientNoOpExtensionFactory()])

    @with_server(
        extensions=[
            # No match because the client doesn't send client_max_window_bits.
            ServerPerMessageDeflateFactory(
                client_max_window_bits=10,
                require_client_max_window_bits=True,
            ),
            ServerPerMessageDeflateFactory(),
        ]
    )
    @with_client(
        "/extensions",
        extensions=[
            ClientPerMessageDeflateFactory(client_max_window_bits=None),
        ],
    )
    def test_extension_no_match_then_match(self):
        # The order requested by the client has priority.
        server_extensions = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(
            server_extensions, repr([PerMessageDeflate(False, False, 15, 15)])
        )
        self.assertEqual(
            repr(self.client.extensions),
            repr([PerMessageDeflate(False, False, 15, 15)]),
        )

    @with_server(extensions=[ServerPerMessageDeflateFactory()])
    @with_client("/extensions", extensions=[ClientNoOpExtensionFactory()])
    def test_extension_mismatch(self):
        server_extensions = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(server_extensions, repr([]))
        self.assertEqual(repr(self.client.extensions), repr([]))

    @with_server(
        extensions=[ServerNoOpExtensionFactory(), ServerPerMessageDeflateFactory()]
    )
    @with_client(
        "/extensions",
        extensions=[ClientPerMessageDeflateFactory(), ClientNoOpExtensionFactory()],
    )
    def test_extension_order(self):
        # The order requested by the client has priority.
        server_extensions = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(
            server_extensions,
            repr([PerMessageDeflate(False, False, 15, 15), NoOpExtension()]),
        )
        self.assertEqual(
            repr(self.client.extensions),
            repr([PerMessageDeflate(False, False, 15, 15), NoOpExtension()]),
        )

    @with_server(extensions=[ServerNoOpExtensionFactory()])
    @unittest.mock.patch.object(WebSocketServerProtocol, "process_extensions")
    def test_extensions_error(self, _process_extensions):
        _process_extensions.return_value = "x-no-op", [NoOpExtension()]

        with self.assertRaises(NegotiationError):
            self.start_client(
                "/extensions", extensions=[ClientPerMessageDeflateFactory()]
            )

    @with_server(extensions=[ServerNoOpExtensionFactory()])
    @unittest.mock.patch.object(WebSocketServerProtocol, "process_extensions")
    def test_extensions_error_no_extensions(self, _process_extensions):
        _process_extensions.return_value = "x-no-op", [NoOpExtension()]

        with self.assertRaises(InvalidHandshake):
            self.start_client("/extensions")

    @with_server(compression="deflate")
    @with_client("/extensions", compression="deflate")
    def test_compression_deflate(self):
        server_extensions = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(
            server_extensions, repr([PerMessageDeflate(False, False, 12, 12)])
        )
        self.assertEqual(
            repr(self.client.extensions),
            repr([PerMessageDeflate(False, False, 12, 12)]),
        )

    def test_compression_unsupported_server(self):
        with self.assertRaises(ValueError):
            self.start_server(compression="xz")

    @with_server()
    def test_compression_unsupported_client(self):
        with self.assertRaises(ValueError):
            self.start_client(compression="xz")

    @with_server()
    @with_client("/subprotocol")
    def test_no_subprotocol(self):
        server_subprotocol = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(server_subprotocol, repr(None))
        self.assertEqual(self.client.subprotocol, None)

    @with_server(subprotocols=["superchat", "chat"])
    @with_client("/subprotocol", subprotocols=["otherchat", "chat"])
    def test_subprotocol(self):
        server_subprotocol = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(server_subprotocol, repr("chat"))
        self.assertEqual(self.client.subprotocol, "chat")

    def test_invalid_subprotocol_server(self):
        with self.assertRaises(TypeError):
            self.start_server(subprotocols="sip")

    @with_server()
    def test_invalid_subprotocol_client(self):
        with self.assertRaises(TypeError):
            self.start_client(subprotocols="sip")

    @with_server(subprotocols=["superchat"])
    @with_client("/subprotocol", subprotocols=["otherchat"])
    def test_subprotocol_not_accepted(self):
        server_subprotocol = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(server_subprotocol, repr(None))
        self.assertEqual(self.client.subprotocol, None)

    @with_server()
    @with_client("/subprotocol", subprotocols=["otherchat", "chat"])
    def test_subprotocol_not_offered(self):
        server_subprotocol = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(server_subprotocol, repr(None))
        self.assertEqual(self.client.subprotocol, None)

    @with_server(subprotocols=["superchat", "chat"])
    @with_client("/subprotocol")
    def test_subprotocol_not_requested(self):
        server_subprotocol = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(server_subprotocol, repr(None))
        self.assertEqual(self.client.subprotocol, None)

    @with_server(subprotocols=["superchat"])
    @unittest.mock.patch.object(WebSocketServerProtocol, "process_subprotocol")
    def test_subprotocol_error(self, _process_subprotocol):
        _process_subprotocol.return_value = "superchat"

        with self.assertRaises(NegotiationError):
            self.start_client("/subprotocol", subprotocols=["otherchat"])
        self.run_loop_once()

    @with_server(subprotocols=["superchat"])
    @unittest.mock.patch.object(WebSocketServerProtocol, "process_subprotocol")
    def test_subprotocol_error_no_subprotocols(self, _process_subprotocol):
        _process_subprotocol.return_value = "superchat"

        with self.assertRaises(InvalidHandshake):
            self.start_client("/subprotocol")
        self.run_loop_once()

    @with_server(subprotocols=["superchat", "chat"])
    @unittest.mock.patch.object(WebSocketServerProtocol, "process_subprotocol")
    def test_subprotocol_error_two_subprotocols(self, _process_subprotocol):
        _process_subprotocol.return_value = "superchat, chat"

        with self.assertRaises(InvalidHandshake):
            self.start_client("/subprotocol", subprotocols=["superchat", "chat"])
        self.run_loop_once()

    @with_server()
    @unittest.mock.patch("websockets.legacy.server.read_request")
    def test_server_receives_malformed_request(self, _read_request):
        _read_request.side_effect = ValueError("read_request failed")

        with self.assertRaises(InvalidHandshake):
            self.start_client()

    @with_server()
    @unittest.mock.patch("websockets.legacy.client.read_response")
    def test_client_receives_malformed_response(self, _read_response):
        _read_response.side_effect = ValueError("read_response failed")

        with self.assertRaises(InvalidHandshake):
            self.start_client()
        self.run_loop_once()

    @with_server()
    @unittest.mock.patch("websockets.legacy.client.build_request")
    def test_client_sends_invalid_handshake_request(self, _build_request):
        def wrong_build_request(headers):
            return "42"

        _build_request.side_effect = wrong_build_request

        with self.assertRaises(InvalidHandshake):
            self.start_client()

    @with_server()
    @unittest.mock.patch("websockets.legacy.server.build_response")
    def test_server_sends_invalid_handshake_response(self, _build_response):
        def wrong_build_response(headers, key):
            return build_response(headers, "42")

        _build_response.side_effect = wrong_build_response

        with self.assertRaises(InvalidHandshake):
            self.start_client()

    @with_server()
    @unittest.mock.patch("websockets.legacy.client.read_response")
    def test_server_does_not_switch_protocols(self, _read_response):
        async def wrong_read_response(stream):
            status_code, reason, headers = await read_response(stream)
            return 400, "Bad Request", headers

        _read_response.side_effect = wrong_read_response

        with self.assertRaises(InvalidStatusCode):
            self.start_client()
        self.run_loop_once()

    @with_server()
    @unittest.mock.patch(
        "websockets.legacy.server.WebSocketServerProtocol.process_request"
    )
    def test_server_error_in_handshake(self, _process_request):
        _process_request.side_effect = Exception("process_request crashed")

        with self.assertRaises(InvalidHandshake):
            self.start_client()

    @with_server(create_protocol=SlowOpeningHandshakeProtocol)
    def test_client_connect_canceled_during_handshake(self):
        sock = socket.create_connection(get_server_address(self.server))
        sock.send(b"")  # socket is connected

        async def cancelled_client():
            start_client = connect(get_server_uri(self.server), sock=sock)
            async with asyncio_timeout(5 * MS):
                await start_client

        with self.assertRaises(asyncio.TimeoutError):
            self.loop.run_until_complete(cancelled_client())

        with self.assertRaises(OSError):
            sock.send(b"")  # socket is closed

    @with_server()
    @unittest.mock.patch("websockets.legacy.server.WebSocketServerProtocol.send")
    def test_server_handler_crashes(self, send):
        send.side_effect = ValueError("send failed")

        with self.temp_client():
            self.loop.run_until_complete(self.client.send("Hello!"))
            with self.assertRaises(ConnectionClosed):
                self.loop.run_until_complete(self.client.recv())

        # Connection ends with an unexpected error.
        self.assertEqual(self.client.close_code, CloseCode.INTERNAL_ERROR)

    @with_server()
    @unittest.mock.patch("websockets.legacy.server.WebSocketServerProtocol.close")
    def test_server_close_crashes(self, close):
        close.side_effect = ValueError("close failed")

        with self.temp_client():
            self.loop.run_until_complete(self.client.send("Hello!"))
            reply = self.loop.run_until_complete(self.client.recv())
            self.assertEqual(reply, "Hello!")

        # Connection ends with an abnormal closure.
        self.assertEqual(self.client.close_code, CloseCode.ABNORMAL_CLOSURE)

    @with_server()
    @with_client()
    @unittest.mock.patch.object(WebSocketClientProtocol, "handshake")
    def test_client_closes_connection_before_handshake(self, handshake):
        # We have mocked the handshake() method to prevent the client from
        # performing the opening handshake. Force it to close the connection.
        self.client.transport.close()
        # The server should stop properly anyway. It used to hang because the
        # task handling the connection was waiting for the opening handshake.

    @with_server(create_protocol=SlowOpeningHandshakeProtocol)
    def test_server_shuts_down_during_opening_handshake(self):
        self.loop.call_later(5 * MS, self.server.close)
        with self.assertRaises(InvalidStatusCode) as raised:
            self.start_client()
        exception = raised.exception
        self.assertEqual(
            str(exception), "server rejected WebSocket connection: HTTP 503"
        )
        self.assertEqual(exception.status_code, 503)

    @with_server()
    def test_server_shuts_down_during_connection_handling(self):
        with self.temp_client():
            server_ws = next(iter(self.server.websockets))
            self.server.close()
            with self.assertRaises(ConnectionClosed):
                self.loop.run_until_complete(self.client.send("Hello!"))
                self.loop.run_until_complete(self.client.recv())

        # Server closed the connection with 1001 Going Away.
        self.assertEqual(self.client.close_code, CloseCode.GOING_AWAY)
        self.assertEqual(server_ws.close_code, CloseCode.GOING_AWAY)

    @with_server()
    def test_server_shuts_down_gracefully_during_connection_handling(self):
        with self.temp_client():
            server_ws = next(iter(self.server.websockets))
            self.server.close(close_connections=False)
            self.loop.run_until_complete(self.client.send("Hello!"))
            self.loop.run_until_complete(self.client.recv())

        # Client closed the connection with 1000 OK.
        self.assertEqual(self.client.close_code, CloseCode.NORMAL_CLOSURE)
        self.assertEqual(server_ws.close_code, CloseCode.NORMAL_CLOSURE)

    @with_server()
    def test_server_shuts_down_and_waits_until_handlers_terminate(self):
        # This handler waits a bit after the connection is closed in order
        # to test that wait_closed() really waits for handlers to complete.
        self.start_client("/slow_stop")
        server_ws = next(iter(self.server.websockets))

        # Test that the handler task keeps running after close().
        self.server.close()
        self.loop.run_until_complete(asyncio.sleep(MS))
        self.assertFalse(server_ws.handler_task.done())

        # Test that the handler task terminates before wait_closed() returns.
        self.loop.run_until_complete(self.server.wait_closed())
        self.assertTrue(server_ws.handler_task.done())

    @with_server(create_protocol=ForbiddenServerProtocol)
    def test_invalid_status_error_during_client_connect(self):
        with self.assertRaises(InvalidStatusCode) as raised:
            self.start_client()
        exception = raised.exception
        self.assertEqual(
            str(exception), "server rejected WebSocket connection: HTTP 403"
        )
        self.assertEqual(exception.status_code, 403)

    @with_server()
    @unittest.mock.patch(
        "websockets.legacy.server.WebSocketServerProtocol.write_http_response"
    )
    @unittest.mock.patch(
        "websockets.legacy.server.WebSocketServerProtocol.read_http_request"
    )
    def test_connection_error_during_opening_handshake(
        self, _read_http_request, _write_http_response
    ):
        _read_http_request.side_effect = ConnectionError

        # This exception is currently platform-dependent. It was observed to
        # be ConnectionResetError on Linux in the non-TLS case, and
        # InvalidMessage otherwise (including both Linux and macOS). This
        # doesn't matter though since this test is primarily for testing a
        # code path on the server side.
        with self.assertRaises(Exception):
            self.start_client()

        # No response must not be written if the network connection is broken.
        _write_http_response.assert_not_called()

    @with_server()
    @unittest.mock.patch("websockets.legacy.server.WebSocketServerProtocol.close")
    def test_connection_error_during_closing_handshake(self, close):
        close.side_effect = ConnectionError

        with self.temp_client():
            self.loop.run_until_complete(self.client.send("Hello!"))
            reply = self.loop.run_until_complete(self.client.recv())
            self.assertEqual(reply, "Hello!")

        # Connection ends with an abnormal closure.
        self.assertEqual(self.client.close_code, CloseCode.ABNORMAL_CLOSURE)


class ClientServerTests(
    CommonClientServerTests, ClientServerTestsMixin, AsyncioTestCase
):
    pass


class SecureClientServerTests(
    CommonClientServerTests, SecureClientServerTestsMixin, AsyncioTestCase
):
    # The implementation of this test makes it hard to run it over TLS.
    test_client_connect_canceled_during_handshake = None

    # TLS over Unix sockets doesn't make sense.
    test_unix_socket = None

    # This test fails under PyPy due to a difference with CPython.
    if platform.python_implementation() == "PyPy":  # pragma: no cover
        test_http_request_ws_endpoint = None

    @with_server()
    def test_ws_uri_is_rejected(self):
        with self.assertRaises(ValueError):
            self.start_client(
                uri=get_server_uri(self.server, secure=False), ssl=self.client_context
            )

    def test_redirect_insecure(self):
        with temp_test_redirecting_server(self):
            with self.assertRaises(InvalidHandshake):
                with self.temp_client("/force_insecure"):
                    self.fail("did not raise")


class ClientServerOriginTests(ClientServerTestsMixin, AsyncioTestCase):
    @with_server(origins=["http://localhost"])
    @with_client(origin="http://localhost")
    def test_checking_origin_succeeds(self):
        self.loop.run_until_complete(self.client.send("Hello!"))
        self.assertEqual(self.loop.run_until_complete(self.client.recv()), "Hello!")

    @with_server(origins=["http://localhost"])
    def test_checking_origin_fails(self):
        with self.assertRaisesRegex(
            InvalidHandshake, "server rejected WebSocket connection: HTTP 403"
        ):
            self.start_client(origin="http://otherhost")

    @with_server(origins=["http://localhost"])
    def test_checking_origins_fails_with_multiple_headers(self):
        with self.assertRaisesRegex(
            InvalidHandshake, "server rejected WebSocket connection: HTTP 400"
        ):
            self.start_client(
                origin="http://localhost",
                extra_headers=[("Origin", "http://otherhost")],
            )

    @with_server(origins=[None])
    @with_client()
    def test_checking_lack_of_origin_succeeds(self):
        self.loop.run_until_complete(self.client.send("Hello!"))
        self.assertEqual(self.loop.run_until_complete(self.client.recv()), "Hello!")

    @with_server(origins=[""])
    # The deprecation warning is raised when a client connects to the server.
    @with_client(deprecation_warnings=["use None instead of '' in origins"])
    def test_checking_lack_of_origin_succeeds_backwards_compatibility(self):
        self.loop.run_until_complete(self.client.send("Hello!"))
        self.assertEqual(self.loop.run_until_complete(self.client.recv()), "Hello!")


@unittest.skipIf(
    sys.version_info[:2] >= (3, 11), "asyncio.coroutine has been removed in Python 3.11"
)
class YieldFromTests(ClientServerTestsMixin, AsyncioTestCase):  # pragma: no cover
    @with_server()
    def test_client(self):
        # @asyncio.coroutine is deprecated on Python ≥ 3.8
        with warnings.catch_warnings():
            warnings.simplefilter("ignore")

            @asyncio.coroutine
            def run_client():
                # Yield from connect.
                client = yield from connect(get_server_uri(self.server))
                self.assertEqual(client.state, State.OPEN)
                yield from client.close()
                self.assertEqual(client.state, State.CLOSED)

        self.loop.run_until_complete(run_client())

    def test_server(self):
        # @asyncio.coroutine is deprecated on Python ≥ 3.8
        with warnings.catch_warnings():
            warnings.simplefilter("ignore")

            @asyncio.coroutine
            def run_server():
                # Yield from serve.
                server = yield from serve(default_handler, "localhost", 0)
                self.assertTrue(server.sockets)
                server.close()
                yield from server.wait_closed()
                self.assertFalse(server.sockets)

        self.loop.run_until_complete(run_server())


class AsyncAwaitTests(ClientServerTestsMixin, AsyncioTestCase):
    @with_server()
    def test_client(self):
        async def run_client():
            # Await connect.
            client = await connect(get_server_uri(self.server))
            self.assertEqual(client.state, State.OPEN)
            await client.close()
            self.assertEqual(client.state, State.CLOSED)

        self.loop.run_until_complete(run_client())

    def test_server(self):
        async def run_server():
            # Await serve.
            server = await serve(default_handler, "localhost", 0)
            self.assertTrue(server.sockets)
            server.close()
            await server.wait_closed()
            self.assertFalse(server.sockets)

        self.loop.run_until_complete(run_server())


class ContextManagerTests(ClientServerTestsMixin, AsyncioTestCase):
    @with_server()
    def test_client(self):
        async def run_client():
            # Use connect as an asynchronous context manager.
            async with connect(get_server_uri(self.server)) as client:
                self.assertEqual(client.state, State.OPEN)

            # Check that exiting the context manager closed the connection.
            self.assertEqual(client.state, State.CLOSED)

        self.loop.run_until_complete(run_client())

    def test_server(self):
        async def run_server():
            # Use serve as an asynchronous context manager.
            async with serve(default_handler, "localhost", 0) as server:
                self.assertTrue(server.sockets)

            # Check that exiting the context manager closed the server.
            self.assertFalse(server.sockets)

        self.loop.run_until_complete(run_server())

    @unittest.skipUnless(hasattr(socket, "AF_UNIX"), "this test requires Unix sockets")
    def test_unix_server(self):
        async def run_server(path):
            async with unix_serve(default_handler, path) as server:
                self.assertTrue(server.sockets)

            # Check that exiting the context manager closed the server.
            self.assertFalse(server.sockets)

        with temp_unix_socket_path() as path:
            self.loop.run_until_complete(run_server(path))


class AsyncIteratorTests(ClientServerTestsMixin, AsyncioTestCase):
    # This is a protocol-level feature, but since it's a high-level API, it is
    # much easier to exercise at the client or server level.

    MESSAGES = ["3", "2", "1", "Fire!"]

    async def echo_handler(ws):
        for message in AsyncIteratorTests.MESSAGES:
            await ws.send(message)

    @with_server(handler=echo_handler)
    def test_iterate_on_messages(self):
        messages = []

        async def run_client():
            nonlocal messages
            async with connect(get_server_uri(self.server)) as ws:
                async for message in ws:
                    messages.append(message)

        self.loop.run_until_complete(run_client())

        self.assertEqual(messages, self.MESSAGES)

    async def echo_handler_going_away(ws):
        for message in AsyncIteratorTests.MESSAGES:
            await ws.send(message)
        await ws.close(CloseCode.GOING_AWAY)

    @with_server(handler=echo_handler_going_away)
    def test_iterate_on_messages_going_away_exit_ok(self):
        messages = []

        async def run_client():
            nonlocal messages
            async with connect(get_server_uri(self.server)) as ws:
                async for message in ws:
                    messages.append(message)

        self.loop.run_until_complete(run_client())

        self.assertEqual(messages, self.MESSAGES)

    async def echo_handler_internal_error(ws):
        for message in AsyncIteratorTests.MESSAGES:
            await ws.send(message)
        await ws.close(CloseCode.INTERNAL_ERROR)

    @with_server(handler=echo_handler_internal_error)
    def test_iterate_on_messages_internal_error_exit_not_ok(self):
        messages = []

        async def run_client():
            nonlocal messages
            async with connect(get_server_uri(self.server)) as ws:
                async for message in ws:
                    messages.append(message)

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(run_client())

        self.assertEqual(messages, self.MESSAGES)


class ReconnectionTests(ClientServerTestsMixin, AsyncioTestCase):
    async def echo_handler(ws):
        async for msg in ws:
            await ws.send(msg)

    service_available = True

    async def maybe_service_unavailable(path, headers):
        if not ReconnectionTests.service_available:
            return http.HTTPStatus.SERVICE_UNAVAILABLE, [], b""

    async def disable_server(self, duration):
        ReconnectionTests.service_available = False
        await asyncio.sleep(duration)
        ReconnectionTests.service_available = True

    @with_server(handler=echo_handler, process_request=maybe_service_unavailable)
    def test_reconnect(self):
        # Big, ugly integration test :-(

        async def run_client():
            iteration = 0
            connect_inst = connect(get_server_uri(self.server))
            connect_inst.BACKOFF_MIN = 10 * MS
            connect_inst.BACKOFF_MAX = 99 * MS
            connect_inst.BACKOFF_INITIAL = 0
            # coverage has a hard time dealing with this code - I give up.
            async for ws in connect_inst:  # pragma: no cover
                await ws.send("spam")
                msg = await ws.recv()
                self.assertEqual(msg, "spam")

                iteration += 1
                if iteration == 1:
                    # Exit block normally.
                    pass
                elif iteration == 2:
                    # Disable server for a little bit
                    asyncio.create_task(self.disable_server(50 * MS))
                    await asyncio.sleep(0)
                elif iteration == 3:
                    # Exit block after catching connection error.
                    server_ws = next(iter(self.server.websockets))
                    await server_ws.close()
                    with self.assertRaises(ConnectionClosed):
                        await ws.recv()
                else:
                    # Exit block with an exception.
                    raise Exception("BOOM")
                pass  # work around bug in coverage

        with self.assertLogs("websockets", logging.INFO) as logs:
            with self.assertRaisesRegex(Exception, "BOOM"):
                self.loop.run_until_complete(run_client())

        # Iteration 1
        self.assertEqual(
            [record.getMessage() for record in logs.records][:2],
            [
                "connection open",
                "connection closed",
            ],
        )
        # Iteration 2
        self.assertEqual(
            [record.getMessage() for record in logs.records][2:4],
            [
                "connection open",
                "connection closed",
            ],
        )
        # Iteration 3
        self.assertEqual(
            [record.getMessage() for record in logs.records][4:-1],
            [
                "connection rejected (503 Service Unavailable)",
                "connection closed",
                "! connect failed; reconnecting in 0.0 seconds",
            ]
            + [
                "connection rejected (503 Service Unavailable)",
                "connection closed",
                "! connect failed again; retrying in 0 seconds",
            ]
            * ((len(logs.records) - 8) // 3)
            + [
                "connection open",
                "connection closed",
            ],
        )
        # Iteration 4
        self.assertEqual(
            [record.getMessage() for record in logs.records][-1:],
            [
                "connection open",
            ],
        )


class LoggerTests(ClientServerTestsMixin, AsyncioTestCase):
    def test_logger_client(self):
        with self.assertLogs("test.server", logging.DEBUG) as server_logs:
            self.start_server(logger=logging.getLogger("test.server"))
            with self.assertLogs("test.client", logging.DEBUG) as client_logs:
                self.start_client(logger=logging.getLogger("test.client"))
                self.loop.run_until_complete(self.client.send("Hello!"))
                self.loop.run_until_complete(self.client.recv())
                self.stop_client()
            self.stop_server()

        self.assertGreater(len(server_logs.records), 0)
        self.assertGreater(len(client_logs.records), 0)
