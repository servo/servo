import unittest

from websockets.exceptions import InvalidURI
from websockets.uri import *


VALID_URIS = [
    (
        "ws://localhost/",
        WebSocketURI(False, "localhost", 80, "/", "", None, None),
    ),
    (
        "wss://localhost/",
        WebSocketURI(True, "localhost", 443, "/", "", None, None),
    ),
    (
        "ws://localhost",
        WebSocketURI(False, "localhost", 80, "", "", None, None),
    ),
    (
        "ws://localhost/path?query",
        WebSocketURI(False, "localhost", 80, "/path", "query", None, None),
    ),
    (
        "ws://localhost/path;params",
        WebSocketURI(False, "localhost", 80, "/path;params", "", None, None),
    ),
    (
        "WS://LOCALHOST/PATH?QUERY",
        WebSocketURI(False, "localhost", 80, "/PATH", "QUERY", None, None),
    ),
    (
        "ws://user:pass@localhost/",
        WebSocketURI(False, "localhost", 80, "/", "", "user", "pass"),
    ),
    (
        "ws://høst/",
        WebSocketURI(False, "xn--hst-0na", 80, "/", "", None, None),
    ),
    (
        "ws://üser:påss@høst/πass?qùéry",
        WebSocketURI(
            False,
            "xn--hst-0na",
            80,
            "/%CF%80ass",
            "q%C3%B9%C3%A9ry",
            "%C3%BCser",
            "p%C3%A5ss",
        ),
    ),
]

INVALID_URIS = [
    "http://localhost/",
    "https://localhost/",
    "ws://localhost/path#fragment",
    "ws://user@localhost/",
    "ws:///path",
]

RESOURCE_NAMES = [
    ("ws://localhost/", "/"),
    ("ws://localhost", "/"),
    ("ws://localhost/path?query", "/path?query"),
    ("ws://høst/πass?qùéry", "/%CF%80ass?q%C3%B9%C3%A9ry"),
]

USER_INFOS = [
    ("ws://localhost/", None),
    ("ws://user:pass@localhost/", ("user", "pass")),
    ("ws://üser:påss@høst/", ("%C3%BCser", "p%C3%A5ss")),
]


class URITests(unittest.TestCase):
    def test_success(self):
        for uri, parsed in VALID_URIS:
            with self.subTest(uri=uri):
                self.assertEqual(parse_uri(uri), parsed)

    def test_error(self):
        for uri in INVALID_URIS:
            with self.subTest(uri=uri):
                with self.assertRaises(InvalidURI):
                    parse_uri(uri)

    def test_resource_name(self):
        for uri, resource_name in RESOURCE_NAMES:
            with self.subTest(uri=uri):
                self.assertEqual(parse_uri(uri).resource_name, resource_name)

    def test_user_info(self):
        for uri, user_info in USER_INFOS:
            with self.subTest(uri=uri):
                self.assertEqual(parse_uri(uri).user_info, user_info)
