import asyncio
import contextlib
import functools
import http
import pathlib
import random
import socket
import ssl
import tempfile
import unittest
import unittest.mock
import urllib.error
import urllib.request
import warnings

from websockets.client import *
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
from websockets.handshake import build_response
from websockets.http import USER_AGENT, Headers, read_response
from websockets.protocol import State
from websockets.server import *
from websockets.uri import parse_uri

from .test_protocol import MS
from .utils import AsyncioTestCase


# Generate TLS certificate with:
# $ openssl req -x509 -config test_localhost.cnf -days 15340 -newkey rsa:2048 \
#       -out test_localhost.crt -keyout test_localhost.key
# $ cat test_localhost.key test_localhost.crt > test_localhost.pem
# $ rm test_localhost.key test_localhost.crt

testcert = bytes(pathlib.Path(__file__).with_name("test_localhost.pem"))


async def handler(ws, path):
    if path == "/deprecated_attributes":
        await ws.recv()  # delay that allows catching warnings
        await ws.send(repr((ws.host, ws.port, ws.secure)))
    elif path == "/close_timeout":
        await ws.send(repr(ws.close_timeout))
    elif path == "/path":
        await ws.send(str(ws.path))
    elif path == "/headers":
        await ws.send(repr(ws.request_headers))
        await ws.send(repr(ws.response_headers))
    elif path == "/extensions":
        await ws.send(repr(ws.extensions))
    elif path == "/subprotocol":
        await ws.send(repr(ws.subprotocol))
    elif path == "/slow_stop":
        await ws.wait_closed()
        await asyncio.sleep(2 * MS)
    else:
        await ws.send((await ws.recv()))


@contextlib.contextmanager
def temp_test_server(test, **kwargs):
    test.start_server(**kwargs)
    try:
        yield
    finally:
        test.stop_server()


@contextlib.contextmanager
def temp_test_redirecting_server(
    test, status, include_location=True, force_insecure=False
):
    test.start_redirecting_server(status, include_location, force_insecure)
    try:
        yield
    finally:
        test.stop_redirecting_server()


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


def get_server_uri(server, secure=False, resource_name="/", user_info=None):
    """
    Return a WebSocket URI for connecting to the given server.

    """
    proto = "wss" if secure else "ws"

    user_info = ":".join(user_info) + "@" if user_info else ""

    # Pick a random socket in order to test both IPv4 and IPv6 on systems
    # where both are available. Randomizing tests is usually a bad idea. If
    # needed, either use the first socket, or test separately IPv4 and IPv6.
    server_socket = random.choice(server.sockets)

    if server_socket.family == socket.AF_INET6:  # pragma: no cover
        host, port = server_socket.getsockname()[:2]  # (no IPv6 on CI)
        host = f"[{host}]"
    elif server_socket.family == socket.AF_INET:
        host, port = server_socket.getsockname()
    else:  # pragma: no cover
        raise ValueError("expected an IPv6, IPv4, or Unix socket")

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


class SlowOpeningHandshakeProtocol(WebSocketServerProtocol):
    async def process_request(self, path, request_headers):
        await asyncio.sleep(10 * MS)


class FooClientProtocol(WebSocketClientProtocol):
    pass


class BarClientProtocol(WebSocketClientProtocol):
    pass


class ClientNoOpExtensionFactory:
    name = "x-no-op"

    def get_request_params(self):
        return []

    def process_response_params(self, params, accepted_extensions):
        if params:
            raise NegotiationError()
        return NoOpExtension()


class ServerNoOpExtensionFactory:
    name = "x-no-op"

    def __init__(self, params=None):
        self.params = params or []

    def process_request_params(self, params, accepted_extensions):
        return self.params, NoOpExtension()


class NoOpExtension:
    name = "x-no-op"

    def __repr__(self):
        return "NoOpExtension()"

    def decode(self, frame, *, max_size=None):
        return frame

    def encode(self, frame):
        return frame


class ClientServerTestsMixin:

    secure = False

    def setUp(self):
        super().setUp()
        self.server = None
        self.redirecting_server = None

    @property
    def server_context(self):
        return None

    def start_server(self, deprecation_warnings=None, **kwargs):
        # Disable compression by default in tests.
        kwargs.setdefault("compression", None)
        # Disable pings by default in tests.
        kwargs.setdefault("ping_interval", None)

        with warnings.catch_warnings(record=True) as recorded_warnings:
            start_server = serve(handler, "localhost", 0, **kwargs)
            self.server = self.loop.run_until_complete(start_server)

        expected_warnings = [] if deprecation_warnings is None else deprecation_warnings
        self.assertDeprecationWarnings(recorded_warnings, expected_warnings)

    def start_redirecting_server(
        self, status, include_location=True, force_insecure=False
    ):
        async def process_request(path, headers):
            server_uri = get_server_uri(self.server, self.secure, path)
            if force_insecure:
                server_uri = server_uri.replace("wss:", "ws:")
            headers = {"Location": server_uri} if include_location else []
            return status, headers, b""

        start_server = serve(
            handler,
            "localhost",
            0,
            compression=None,
            ping_interval=None,
            process_request=process_request,
            ssl=self.server_context,
        )
        self.redirecting_server = self.loop.run_until_complete(start_server)

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
            server = self.redirecting_server if self.redirecting_server else self.server
            server_uri = get_server_uri(server, secure, resource_name, user_info)

        with warnings.catch_warnings(record=True) as recorded_warnings:
            start_client = connect(server_uri, **kwargs)
            self.client = self.loop.run_until_complete(start_client)

        expected_warnings = [] if deprecation_warnings is None else deprecation_warnings
        self.assertDeprecationWarnings(recorded_warnings, expected_warnings)

    def stop_client(self):
        try:
            self.loop.run_until_complete(
                asyncio.wait_for(self.client.close_connection_task, timeout=1)
            )
        except asyncio.TimeoutError:  # pragma: no cover
            self.fail("Client failed to stop")

    def stop_server(self):
        self.server.close()
        try:
            self.loop.run_until_complete(
                asyncio.wait_for(self.server.wait_closed(), timeout=1)
            )
        except asyncio.TimeoutError:  # pragma: no cover
            self.fail("Server failed to stop")

    def stop_redirecting_server(self):
        self.redirecting_server.close()
        try:
            self.loop.run_until_complete(
                asyncio.wait_for(self.redirecting_server.wait_closed(), timeout=1)
            )
        except asyncio.TimeoutError:  # pragma: no cover
            self.fail("Redirecting server failed to stop")
        finally:
            self.redirecting_server = None

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
        ssl_context.load_cert_chain(testcert)
        return ssl_context

    @property
    def client_context(self):
        ssl_context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
        ssl_context.load_verify_locations(testcert)
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

    @with_server()
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
                with temp_test_client(self):
                    self.loop.run_until_complete(self.client.send("Hello!"))
                    reply = self.loop.run_until_complete(self.client.recv())
                    self.assertEqual(reply, "Hello!")

    def test_infinite_redirect(self):
        with temp_test_redirecting_server(self, http.HTTPStatus.FOUND):
            self.server = self.redirecting_server
            with self.assertRaises(InvalidHandshake):
                with temp_test_client(self):
                    self.fail("Did not raise")  # pragma: no cover

    @with_server()
    def test_redirect_missing_location(self):
        with temp_test_redirecting_server(
            self, http.HTTPStatus.FOUND, include_location=False
        ):
            with self.assertRaises(InvalidHeader):
                with temp_test_client(self):
                    self.fail("Did not raise")  # pragma: no cover

    def test_explicit_event_loop(self):
        with self.temp_server(loop=self.loop):
            with self.temp_client(loop=self.loop):
                self.loop.run_until_complete(self.client.send("Hello!"))
                reply = self.loop.run_until_complete(self.client.recv())
                self.assertEqual(reply, "Hello!")

    @with_server()
    def test_explicit_host_port(self):
        uri = get_server_uri(self.server, self.secure)
        wsuri = parse_uri(uri)

        # Change host and port to invalid values.
        changed_uri = uri.replace(wsuri.host, "example.com").replace(
            str(wsuri.port), str(65535 - wsuri.port)
        )

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

            with self.temp_client(
                sock=client_socket,
                # "You must set server_hostname when using ssl without a host"
                server_hostname="localhost" if self.secure else None,
            ):
                self.loop.run_until_complete(self.client.send("Hello!"))
                reply = self.loop.run_until_complete(self.client.recv())
                self.assertEqual(reply, "Hello!")

            self.assertTrue(client_socket.used_for_read)
            self.assertTrue(client_socket.used_for_write)

        finally:
            client_socket.close()

    @unittest.skipUnless(hasattr(socket, "AF_UNIX"), "this test requires Unix sockets")
    def test_unix_socket(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            path = bytes(pathlib.Path(temp_dir) / "websockets")

            # Like self.start_server() but with unix_serve().
            unix_server = unix_serve(handler, path)
            self.server = self.loop.run_until_complete(unix_server)
            try:
                # Like self.start_client() but with unix_connect()
                unix_client = unix_connect(path)
                self.client = self.loop.run_until_complete(unix_client)
                try:
                    self.loop.run_until_complete(self.client.send("Hello!"))
                    reply = self.loop.run_until_complete(self.client.recv())
                    self.assertEqual(reply, "Hello!")
                finally:
                    self.stop_client()
            finally:
                self.stop_server()

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
    @with_client("/headers", extra_headers=Headers({"X-Spam": "Eggs"}))
    def test_protocol_custom_request_headers(self):
        req_headers = self.loop.run_until_complete(self.client.recv())
        self.loop.run_until_complete(self.client.recv())
        self.assertIn("('X-Spam', 'Eggs')", req_headers)

    @with_server()
    @with_client("/headers", extra_headers={"X-Spam": "Eggs"})
    def test_protocol_custom_request_headers_dict(self):
        req_headers = self.loop.run_until_complete(self.client.recv())
        self.loop.run_until_complete(self.client.recv())
        self.assertIn("('X-Spam', 'Eggs')", req_headers)

    @with_server()
    @with_client("/headers", extra_headers=[("X-Spam", "Eggs")])
    def test_protocol_custom_request_headers_list(self):
        req_headers = self.loop.run_until_complete(self.client.recv())
        self.loop.run_until_complete(self.client.recv())
        self.assertIn("('X-Spam', 'Eggs')", req_headers)

    @with_server()
    @with_client("/headers", extra_headers=[("User-Agent", "Eggs")])
    def test_protocol_custom_request_user_agent(self):
        req_headers = self.loop.run_until_complete(self.client.recv())
        self.loop.run_until_complete(self.client.recv())
        self.assertEqual(req_headers.count("User-Agent"), 1)
        self.assertIn("('User-Agent', 'Eggs')", req_headers)

    @with_server(extra_headers=lambda p, r: Headers({"X-Spam": "Eggs"}))
    @with_client("/headers")
    def test_protocol_custom_response_headers_callable(self):
        self.loop.run_until_complete(self.client.recv())
        resp_headers = self.loop.run_until_complete(self.client.recv())
        self.assertIn("('X-Spam', 'Eggs')", resp_headers)

    @with_server(extra_headers=lambda p, r: {"X-Spam": "Eggs"})
    @with_client("/headers")
    def test_protocol_custom_response_headers_callable_dict(self):
        self.loop.run_until_complete(self.client.recv())
        resp_headers = self.loop.run_until_complete(self.client.recv())
        self.assertIn("('X-Spam', 'Eggs')", resp_headers)

    @with_server(extra_headers=lambda p, r: [("X-Spam", "Eggs")])
    @with_client("/headers")
    def test_protocol_custom_response_headers_callable_list(self):
        self.loop.run_until_complete(self.client.recv())
        resp_headers = self.loop.run_until_complete(self.client.recv())
        self.assertIn("('X-Spam', 'Eggs')", resp_headers)

    @with_server(extra_headers=lambda p, r: None)
    @with_client("/headers")
    def test_protocol_custom_response_headers_callable_none(self):
        self.loop.run_until_complete(self.client.recv())  # doesn't crash
        self.loop.run_until_complete(self.client.recv())  # nothing to check

    @with_server(extra_headers=Headers({"X-Spam": "Eggs"}))
    @with_client("/headers")
    def test_protocol_custom_response_headers(self):
        self.loop.run_until_complete(self.client.recv())
        resp_headers = self.loop.run_until_complete(self.client.recv())
        self.assertIn("('X-Spam', 'Eggs')", resp_headers)

    @with_server(extra_headers={"X-Spam": "Eggs"})
    @with_client("/headers")
    def test_protocol_custom_response_headers_dict(self):
        self.loop.run_until_complete(self.client.recv())
        resp_headers = self.loop.run_until_complete(self.client.recv())
        self.assertIn("('X-Spam', 'Eggs')", resp_headers)

    @with_server(extra_headers=[("X-Spam", "Eggs")])
    @with_client("/headers")
    def test_protocol_custom_response_headers_list(self):
        self.loop.run_until_complete(self.client.recv())
        resp_headers = self.loop.run_until_complete(self.client.recv())
        self.assertIn("('X-Spam', 'Eggs')", resp_headers)

    @with_server(extra_headers=[("Server", "Eggs")])
    @with_client("/headers")
    def test_protocol_custom_response_user_agent(self):
        self.loop.run_until_complete(self.client.recv())
        resp_headers = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(resp_headers.count("Server"), 1)
        self.assertIn("('Server', 'Eggs')", resp_headers)

    @with_server(create_protocol=HealthCheckServerProtocol)
    def test_http_request_http_endpoint(self):
        # Making a HTTP request to a HTTP endpoint succeeds.
        response = self.loop.run_until_complete(self.make_http_request("/__health__/"))

        with contextlib.closing(response):
            self.assertEqual(response.code, 200)
            self.assertEqual(response.read(), b"status = green\n")

    @with_server(create_protocol=HealthCheckServerProtocol)
    def test_http_request_ws_endpoint(self):
        # Making a HTTP request to a WS endpoint fails.
        with self.assertRaises(urllib.error.HTTPError) as raised:
            self.loop.run_until_complete(self.make_http_request())

        self.assertEqual(raised.exception.code, 426)
        self.assertEqual(raised.exception.headers["Upgrade"], "websocket")

    @with_server(create_protocol=HealthCheckServerProtocol)
    def test_ws_connection_http_endpoint(self):
        # Making a WS connection to a HTTP endpoint fails.
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
            ServerPerMessageDeflateFactory(client_max_window_bits=10),
            ServerPerMessageDeflateFactory(),
        ]
    )
    @with_client("/extensions", extensions=[ClientPerMessageDeflateFactory()])
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
            server_extensions, repr([PerMessageDeflate(False, False, 15, 15)])
        )
        self.assertEqual(
            repr(self.client.extensions),
            repr([PerMessageDeflate(False, False, 15, 15)]),
        )

    @with_server(
        extensions=[
            ServerPerMessageDeflateFactory(
                client_no_context_takeover=True, server_max_window_bits=10
            )
        ],
        compression="deflate",  # overridden by explicit config
    )
    @with_client(
        "/extensions",
        extensions=[
            ClientPerMessageDeflateFactory(
                server_no_context_takeover=True, client_max_window_bits=12
            )
        ],
        compression="deflate",  # overridden by explicit config
    )
    def test_compression_deflate_and_explicit_config(self):
        server_extensions = self.loop.run_until_complete(self.client.recv())
        self.assertEqual(
            server_extensions, repr([PerMessageDeflate(True, True, 12, 10)])
        )
        self.assertEqual(
            repr(self.client.extensions), repr([PerMessageDeflate(True, True, 10, 12)])
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
    @unittest.mock.patch("websockets.server.read_request")
    def test_server_receives_malformed_request(self, _read_request):
        _read_request.side_effect = ValueError("read_request failed")

        with self.assertRaises(InvalidHandshake):
            self.start_client()

    @with_server()
    @unittest.mock.patch("websockets.client.read_response")
    def test_client_receives_malformed_response(self, _read_response):
        _read_response.side_effect = ValueError("read_response failed")

        with self.assertRaises(InvalidHandshake):
            self.start_client()
        self.run_loop_once()

    @with_server()
    @unittest.mock.patch("websockets.client.build_request")
    def test_client_sends_invalid_handshake_request(self, _build_request):
        def wrong_build_request(headers):
            return "42"

        _build_request.side_effect = wrong_build_request

        with self.assertRaises(InvalidHandshake):
            self.start_client()

    @with_server()
    @unittest.mock.patch("websockets.server.build_response")
    def test_server_sends_invalid_handshake_response(self, _build_response):
        def wrong_build_response(headers, key):
            return build_response(headers, "42")

        _build_response.side_effect = wrong_build_response

        with self.assertRaises(InvalidHandshake):
            self.start_client()

    @with_server()
    @unittest.mock.patch("websockets.client.read_response")
    def test_server_does_not_switch_protocols(self, _read_response):
        async def wrong_read_response(stream):
            status_code, reason, headers = await read_response(stream)
            return 400, "Bad Request", headers

        _read_response.side_effect = wrong_read_response

        with self.assertRaises(InvalidStatusCode):
            self.start_client()
        self.run_loop_once()

    @with_server()
    @unittest.mock.patch("websockets.server.WebSocketServerProtocol.process_request")
    def test_server_error_in_handshake(self, _process_request):
        _process_request.side_effect = Exception("process_request crashed")

        with self.assertRaises(InvalidHandshake):
            self.start_client()

    @with_server()
    @unittest.mock.patch("websockets.server.WebSocketServerProtocol.send")
    def test_server_handler_crashes(self, send):
        send.side_effect = ValueError("send failed")

        with self.temp_client():
            self.loop.run_until_complete(self.client.send("Hello!"))
            with self.assertRaises(ConnectionClosed):
                self.loop.run_until_complete(self.client.recv())

        # Connection ends with an unexpected error.
        self.assertEqual(self.client.close_code, 1011)

    @with_server()
    @unittest.mock.patch("websockets.server.WebSocketServerProtocol.close")
    def test_server_close_crashes(self, close):
        close.side_effect = ValueError("close failed")

        with self.temp_client():
            self.loop.run_until_complete(self.client.send("Hello!"))
            reply = self.loop.run_until_complete(self.client.recv())
            self.assertEqual(reply, "Hello!")

        # Connection ends with an abnormal closure.
        self.assertEqual(self.client.close_code, 1006)

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
                self.loop.run_until_complete(self.client.recv())

        # Websocket connection closes properly with 1001 Going Away.
        self.assertEqual(self.client.close_code, 1001)
        self.assertEqual(server_ws.close_code, 1001)

    @with_server()
    def test_server_shuts_down_waits_until_handlers_terminate(self):
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
        "websockets.server.WebSocketServerProtocol.write_http_response"
    )
    @unittest.mock.patch("websockets.server.WebSocketServerProtocol.read_http_request")
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
    @unittest.mock.patch("websockets.server.WebSocketServerProtocol.close")
    def test_connection_error_during_closing_handshake(self, close):
        close.side_effect = ConnectionError

        with self.temp_client():
            self.loop.run_until_complete(self.client.send("Hello!"))
            reply = self.loop.run_until_complete(self.client.recv())
            self.assertEqual(reply, "Hello!")

        # Connection ends with an abnormal closure.
        self.assertEqual(self.client.close_code, 1006)


class ClientServerTests(
    CommonClientServerTests, ClientServerTestsMixin, AsyncioTestCase
):
    pass


class SecureClientServerTests(
    CommonClientServerTests, SecureClientServerTestsMixin, AsyncioTestCase
):

    # TLS over Unix sockets doesn't make sense.
    test_unix_socket = None

    @with_server()
    def test_ws_uri_is_rejected(self):
        with self.assertRaises(ValueError):
            connect(get_server_uri(self.server, secure=False), ssl=self.client_context)

    @with_server()
    def test_redirect_insecure(self):
        with temp_test_redirecting_server(
            self, http.HTTPStatus.FOUND, force_insecure=True
        ):
            with self.assertRaises(InvalidHandshake):
                with temp_test_client(self):
                    self.fail("Did not raise")  # pragma: no cover


class ClientServerOriginTests(AsyncioTestCase):
    def test_checking_origin_succeeds(self):
        server = self.loop.run_until_complete(
            serve(handler, "localhost", 0, origins=["http://localhost"])
        )
        client = self.loop.run_until_complete(
            connect(get_server_uri(server), origin="http://localhost")
        )

        self.loop.run_until_complete(client.send("Hello!"))
        self.assertEqual(self.loop.run_until_complete(client.recv()), "Hello!")

        self.loop.run_until_complete(client.close())
        server.close()
        self.loop.run_until_complete(server.wait_closed())

    def test_checking_origin_fails(self):
        server = self.loop.run_until_complete(
            serve(handler, "localhost", 0, origins=["http://localhost"])
        )
        with self.assertRaisesRegex(
            InvalidHandshake, "server rejected WebSocket connection: HTTP 403"
        ):
            self.loop.run_until_complete(
                connect(get_server_uri(server), origin="http://otherhost")
            )

        server.close()
        self.loop.run_until_complete(server.wait_closed())

    def test_checking_origins_fails_with_multiple_headers(self):
        server = self.loop.run_until_complete(
            serve(handler, "localhost", 0, origins=["http://localhost"])
        )
        with self.assertRaisesRegex(
            InvalidHandshake, "server rejected WebSocket connection: HTTP 400"
        ):
            self.loop.run_until_complete(
                connect(
                    get_server_uri(server),
                    origin="http://localhost",
                    extra_headers=[("Origin", "http://otherhost")],
                )
            )

        server.close()
        self.loop.run_until_complete(server.wait_closed())

    def test_checking_lack_of_origin_succeeds(self):
        server = self.loop.run_until_complete(
            serve(handler, "localhost", 0, origins=[None])
        )
        client = self.loop.run_until_complete(connect(get_server_uri(server)))

        self.loop.run_until_complete(client.send("Hello!"))
        self.assertEqual(self.loop.run_until_complete(client.recv()), "Hello!")

        self.loop.run_until_complete(client.close())
        server.close()
        self.loop.run_until_complete(server.wait_closed())

    def test_checking_lack_of_origin_succeeds_backwards_compatibility(self):
        with warnings.catch_warnings(record=True) as recorded_warnings:
            server = self.loop.run_until_complete(
                serve(handler, "localhost", 0, origins=[""])
            )
            client = self.loop.run_until_complete(connect(get_server_uri(server)))

        self.assertDeprecationWarnings(
            recorded_warnings, ["use None instead of '' in origins"]
        )

        self.loop.run_until_complete(client.send("Hello!"))
        self.assertEqual(self.loop.run_until_complete(client.recv()), "Hello!")

        self.loop.run_until_complete(client.close())
        server.close()
        self.loop.run_until_complete(server.wait_closed())


class YieldFromTests(AsyncioTestCase):
    def test_client(self):
        start_server = serve(handler, "localhost", 0)
        server = self.loop.run_until_complete(start_server)

        # @asyncio.coroutine is deprecated on Python ≥ 3.8
        with warnings.catch_warnings(record=True):

            @asyncio.coroutine
            def run_client():
                # Yield from connect.
                client = yield from connect(get_server_uri(server))
                self.assertEqual(client.state, State.OPEN)
                yield from client.close()
                self.assertEqual(client.state, State.CLOSED)

        self.loop.run_until_complete(run_client())

        server.close()
        self.loop.run_until_complete(server.wait_closed())

    def test_server(self):
        # @asyncio.coroutine is deprecated on Python ≥ 3.8
        with warnings.catch_warnings(record=True):

            @asyncio.coroutine
            def run_server():
                # Yield from serve.
                server = yield from serve(handler, "localhost", 0)
                self.assertTrue(server.sockets)
                server.close()
                yield from server.wait_closed()
                self.assertFalse(server.sockets)

        self.loop.run_until_complete(run_server())


class AsyncAwaitTests(AsyncioTestCase):
    def test_client(self):
        start_server = serve(handler, "localhost", 0)
        server = self.loop.run_until_complete(start_server)

        async def run_client():
            # Await connect.
            client = await connect(get_server_uri(server))
            self.assertEqual(client.state, State.OPEN)
            await client.close()
            self.assertEqual(client.state, State.CLOSED)

        self.loop.run_until_complete(run_client())

        server.close()
        self.loop.run_until_complete(server.wait_closed())

    def test_server(self):
        async def run_server():
            # Await serve.
            server = await serve(handler, "localhost", 0)
            self.assertTrue(server.sockets)
            server.close()
            await server.wait_closed()
            self.assertFalse(server.sockets)

        self.loop.run_until_complete(run_server())


class ContextManagerTests(AsyncioTestCase):
    def test_client(self):
        start_server = serve(handler, "localhost", 0)
        server = self.loop.run_until_complete(start_server)

        async def run_client():
            # Use connect as an asynchronous context manager.
            async with connect(get_server_uri(server)) as client:
                self.assertEqual(client.state, State.OPEN)

            # Check that exiting the context manager closed the connection.
            self.assertEqual(client.state, State.CLOSED)

        self.loop.run_until_complete(run_client())

        server.close()
        self.loop.run_until_complete(server.wait_closed())

    def test_server(self):
        async def run_server():
            # Use serve as an asynchronous context manager.
            async with serve(handler, "localhost", 0) as server:
                self.assertTrue(server.sockets)

            # Check that exiting the context manager closed the server.
            self.assertFalse(server.sockets)

        self.loop.run_until_complete(run_server())

    @unittest.skipUnless(hasattr(socket, "AF_UNIX"), "this test requires Unix sockets")
    def test_unix_server(self):
        async def run_server(path):
            async with unix_serve(handler, path) as server:
                self.assertTrue(server.sockets)

            # Check that exiting the context manager closed the server.
            self.assertFalse(server.sockets)

        with tempfile.TemporaryDirectory() as temp_dir:
            path = bytes(pathlib.Path(temp_dir) / "websockets")
            self.loop.run_until_complete(run_server(path))


class AsyncIteratorTests(AsyncioTestCase):

    # This is a protocol-level feature, but since it's a high-level API, it is
    # much easier to exercise at the client or server level.

    MESSAGES = ["3", "2", "1", "Fire!"]

    def test_iterate_on_messages(self):
        async def handler(ws, path):
            for message in self.MESSAGES:
                await ws.send(message)

        start_server = serve(handler, "localhost", 0)
        server = self.loop.run_until_complete(start_server)

        messages = []

        async def run_client():
            nonlocal messages
            async with connect(get_server_uri(server)) as ws:
                async for message in ws:
                    messages.append(message)

        self.loop.run_until_complete(run_client())

        self.assertEqual(messages, self.MESSAGES)

        server.close()
        self.loop.run_until_complete(server.wait_closed())

    def test_iterate_on_messages_going_away_exit_ok(self):
        async def handler(ws, path):
            for message in self.MESSAGES:
                await ws.send(message)
            await ws.close(1001)

        start_server = serve(handler, "localhost", 0)
        server = self.loop.run_until_complete(start_server)

        messages = []

        async def run_client():
            nonlocal messages
            async with connect(get_server_uri(server)) as ws:
                async for message in ws:
                    messages.append(message)

        self.loop.run_until_complete(run_client())

        self.assertEqual(messages, self.MESSAGES)

        server.close()
        self.loop.run_until_complete(server.wait_closed())

    def test_iterate_on_messages_internal_error_exit_not_ok(self):
        async def handler(ws, path):
            for message in self.MESSAGES:
                await ws.send(message)
            await ws.close(1011)

        start_server = serve(handler, "localhost", 0)
        server = self.loop.run_until_complete(start_server)

        messages = []

        async def run_client():
            nonlocal messages
            async with connect(get_server_uri(server)) as ws:
                async for message in ws:
                    messages.append(message)

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(run_client())

        self.assertEqual(messages, self.MESSAGES)

        server.close()
        self.loop.run_until_complete(server.wait_closed())
