import hmac
import unittest
import urllib.error

from websockets.exceptions import InvalidStatusCode
from websockets.headers import build_authorization_basic
from websockets.legacy.auth import *
from websockets.legacy.auth import is_credentials

from .test_client_server import ClientServerTestsMixin, with_client, with_server
from .utils import AsyncioTestCase


class AuthTests(unittest.TestCase):
    def test_is_credentials(self):
        self.assertTrue(is_credentials(("username", "password")))

    def test_is_not_credentials(self):
        self.assertFalse(is_credentials(None))
        self.assertFalse(is_credentials("username"))


class CustomWebSocketServerProtocol(BasicAuthWebSocketServerProtocol):
    async def process_request(self, path, request_headers):
        type(self).used = True
        return await super().process_request(path, request_headers)


class CheckWebSocketServerProtocol(BasicAuthWebSocketServerProtocol):
    async def check_credentials(self, username, password):
        return hmac.compare_digest(password, "letmein")


class AuthClientServerTests(ClientServerTestsMixin, AsyncioTestCase):
    create_protocol = basic_auth_protocol_factory(
        realm="auth-tests", credentials=("hello", "iloveyou")
    )

    @with_server(create_protocol=create_protocol)
    @with_client(user_info=("hello", "iloveyou"))
    def test_basic_auth(self):
        req_headers = self.client.request_headers
        resp_headers = self.client.response_headers
        self.assertEqual(req_headers["Authorization"], "Basic aGVsbG86aWxvdmV5b3U=")
        self.assertNotIn("WWW-Authenticate", resp_headers)

        self.loop.run_until_complete(self.client.send("Hello!"))
        self.loop.run_until_complete(self.client.recv())

    def test_basic_auth_server_no_credentials(self):
        with self.assertRaises(TypeError) as raised:
            basic_auth_protocol_factory(realm="auth-tests", credentials=None)
        self.assertEqual(
            str(raised.exception), "provide either credentials or check_credentials"
        )

    def test_basic_auth_server_bad_credentials(self):
        with self.assertRaises(TypeError) as raised:
            basic_auth_protocol_factory(realm="auth-tests", credentials=42)
        self.assertEqual(str(raised.exception), "invalid credentials argument: 42")

    create_protocol_multiple_credentials = basic_auth_protocol_factory(
        realm="auth-tests",
        credentials=[("hello", "iloveyou"), ("goodbye", "stillloveu")],
    )

    @with_server(create_protocol=create_protocol_multiple_credentials)
    @with_client(user_info=("hello", "iloveyou"))
    def test_basic_auth_server_multiple_credentials(self):
        self.loop.run_until_complete(self.client.send("Hello!"))
        self.loop.run_until_complete(self.client.recv())

    def test_basic_auth_bad_multiple_credentials(self):
        with self.assertRaises(TypeError) as raised:
            basic_auth_protocol_factory(
                realm="auth-tests", credentials=[("hello", "iloveyou"), 42]
            )
        self.assertEqual(
            str(raised.exception),
            "invalid credentials argument: [('hello', 'iloveyou'), 42]",
        )

    async def check_credentials(username, password):
        return hmac.compare_digest(password, "iloveyou")

    create_protocol_check_credentials = basic_auth_protocol_factory(
        realm="auth-tests",
        check_credentials=check_credentials,
    )

    @with_server(create_protocol=create_protocol_check_credentials)
    @with_client(user_info=("hello", "iloveyou"))
    def test_basic_auth_check_credentials(self):
        self.loop.run_until_complete(self.client.send("Hello!"))
        self.loop.run_until_complete(self.client.recv())

    create_protocol_custom_protocol = basic_auth_protocol_factory(
        realm="auth-tests",
        credentials=[("hello", "iloveyou")],
        create_protocol=CustomWebSocketServerProtocol,
    )

    @with_server(create_protocol=create_protocol_custom_protocol)
    @with_client(user_info=("hello", "iloveyou"))
    def test_basic_auth_custom_protocol(self):
        self.assertTrue(CustomWebSocketServerProtocol.used)
        del CustomWebSocketServerProtocol.used
        self.loop.run_until_complete(self.client.send("Hello!"))
        self.loop.run_until_complete(self.client.recv())

    @with_server(create_protocol=CheckWebSocketServerProtocol)
    @with_client(user_info=("hello", "letmein"))
    def test_basic_auth_custom_protocol_subclass(self):
        self.loop.run_until_complete(self.client.send("Hello!"))
        self.loop.run_until_complete(self.client.recv())

    # CustomWebSocketServerProtocol doesn't override check_credentials
    @with_server(create_protocol=CustomWebSocketServerProtocol)
    def test_basic_auth_defaults_to_deny_all(self):
        with self.assertRaises(InvalidStatusCode) as raised:
            self.start_client(user_info=("hello", "iloveyou"))
        self.assertEqual(raised.exception.status_code, 401)

    @with_server(create_protocol=create_protocol)
    def test_basic_auth_missing_credentials(self):
        with self.assertRaises(InvalidStatusCode) as raised:
            self.start_client()
        self.assertEqual(raised.exception.status_code, 401)

    @with_server(create_protocol=create_protocol)
    def test_basic_auth_missing_credentials_details(self):
        with self.assertRaises(urllib.error.HTTPError) as raised:
            self.loop.run_until_complete(self.make_http_request())
        self.assertEqual(raised.exception.code, 401)
        self.assertEqual(
            raised.exception.headers["WWW-Authenticate"],
            'Basic realm="auth-tests", charset="UTF-8"',
        )
        self.assertEqual(raised.exception.read().decode(), "Missing credentials\n")

    @with_server(create_protocol=create_protocol)
    def test_basic_auth_unsupported_credentials(self):
        with self.assertRaises(InvalidStatusCode) as raised:
            self.start_client(extra_headers={"Authorization": "Digest ..."})
        self.assertEqual(raised.exception.status_code, 401)

    @with_server(create_protocol=create_protocol)
    def test_basic_auth_unsupported_credentials_details(self):
        with self.assertRaises(urllib.error.HTTPError) as raised:
            self.loop.run_until_complete(
                self.make_http_request(headers={"Authorization": "Digest ..."})
            )
        self.assertEqual(raised.exception.code, 401)
        self.assertEqual(
            raised.exception.headers["WWW-Authenticate"],
            'Basic realm="auth-tests", charset="UTF-8"',
        )
        self.assertEqual(raised.exception.read().decode(), "Unsupported credentials\n")

    @with_server(create_protocol=create_protocol)
    def test_basic_auth_invalid_username(self):
        with self.assertRaises(InvalidStatusCode) as raised:
            self.start_client(user_info=("goodbye", "iloveyou"))
        self.assertEqual(raised.exception.status_code, 401)

    @with_server(create_protocol=create_protocol)
    def test_basic_auth_invalid_password(self):
        with self.assertRaises(InvalidStatusCode) as raised:
            self.start_client(user_info=("hello", "ihateyou"))
        self.assertEqual(raised.exception.status_code, 401)

    @with_server(create_protocol=create_protocol)
    def test_basic_auth_invalid_credentials_details(self):
        with self.assertRaises(urllib.error.HTTPError) as raised:
            authorization = build_authorization_basic("hello", "ihateyou")
            self.loop.run_until_complete(
                self.make_http_request(headers={"Authorization": authorization})
            )
        self.assertEqual(raised.exception.code, 401)
        self.assertEqual(
            raised.exception.headers["WWW-Authenticate"],
            'Basic realm="auth-tests", charset="UTF-8"',
        )
        self.assertEqual(raised.exception.read().decode(), "Invalid credentials\n")
