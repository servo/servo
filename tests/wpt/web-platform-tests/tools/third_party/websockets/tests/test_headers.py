import unittest

from websockets.exceptions import InvalidHeaderFormat, InvalidHeaderValue
from websockets.headers import *


class HeadersTests(unittest.TestCase):
    def test_parse_connection(self):
        for header, parsed in [
            # Realistic use cases
            ("Upgrade", ["Upgrade"]),  # Safari, Chrome
            ("keep-alive, Upgrade", ["keep-alive", "Upgrade"]),  # Firefox
            # Pathological example
            (",,\t,  , ,Upgrade  ,,", ["Upgrade"]),
        ]:
            with self.subTest(header=header):
                self.assertEqual(parse_connection(header), parsed)

    def test_parse_connection_invalid_header_format(self):
        for header in ["???", "keep-alive; Upgrade"]:
            with self.subTest(header=header):
                with self.assertRaises(InvalidHeaderFormat):
                    parse_connection(header)

    def test_parse_upgrade(self):
        for header, parsed in [
            # Realistic use case
            ("websocket", ["websocket"]),
            # Synthetic example
            ("http/3.0, websocket", ["http/3.0", "websocket"]),
            # Pathological example
            (",,  WebSocket,  \t,,", ["WebSocket"]),
        ]:
            with self.subTest(header=header):
                self.assertEqual(parse_upgrade(header), parsed)

    def test_parse_upgrade_invalid_header_format(self):
        for header in ["???", "websocket 2", "http/3.0; websocket"]:
            with self.subTest(header=header):
                with self.assertRaises(InvalidHeaderFormat):
                    parse_upgrade(header)

    def test_parse_extension(self):
        for header, parsed in [
            # Synthetic examples
            ("foo", [("foo", [])]),
            ("foo, bar", [("foo", []), ("bar", [])]),
            (
                'foo; name; token=token; quoted-string="quoted-string", '
                "bar; quux; quuux",
                [
                    (
                        "foo",
                        [
                            ("name", None),
                            ("token", "token"),
                            ("quoted-string", "quoted-string"),
                        ],
                    ),
                    ("bar", [("quux", None), ("quuux", None)]),
                ],
            ),
            # Pathological example
            (
                ",\t, ,  ,foo  ;bar = 42,,   baz,,",
                [("foo", [("bar", "42")]), ("baz", [])],
            ),
            # Realistic use cases for permessage-deflate
            ("permessage-deflate", [("permessage-deflate", [])]),
            (
                "permessage-deflate; client_max_window_bits",
                [("permessage-deflate", [("client_max_window_bits", None)])],
            ),
            (
                "permessage-deflate; server_max_window_bits=10",
                [("permessage-deflate", [("server_max_window_bits", "10")])],
            ),
        ]:
            with self.subTest(header=header):
                self.assertEqual(parse_extension(header), parsed)
                # Also ensure that build_extension round-trips cleanly.
                unparsed = build_extension(parsed)
                self.assertEqual(parse_extension(unparsed), parsed)

    def test_parse_extension_invalid_header_format(self):
        for header in [
            # Truncated examples
            "",
            ",\t,",
            "foo;",
            "foo; bar;",
            "foo; bar=",
            'foo; bar="baz',
            # Wrong delimiter
            "foo, bar, baz=quux; quuux",
            # Value in quoted string parameter that isn't a token
            'foo; bar=" "',
        ]:
            with self.subTest(header=header):
                with self.assertRaises(InvalidHeaderFormat):
                    parse_extension(header)

    def test_parse_subprotocol(self):
        for header, parsed in [
            # Synthetic examples
            ("foo", ["foo"]),
            ("foo, bar", ["foo", "bar"]),
            # Pathological example
            (",\t, ,  ,foo  ,,   bar,baz,,", ["foo", "bar", "baz"]),
        ]:
            with self.subTest(header=header):
                self.assertEqual(parse_subprotocol(header), parsed)
                # Also ensure that build_subprotocol round-trips cleanly.
                unparsed = build_subprotocol(parsed)
                self.assertEqual(parse_subprotocol(unparsed), parsed)

    def test_parse_subprotocol_invalid_header(self):
        for header in [
            # Truncated examples
            "",
            ",\t,"
            # Wrong delimiter
            "foo; bar",
        ]:
            with self.subTest(header=header):
                with self.assertRaises(InvalidHeaderFormat):
                    parse_subprotocol(header)

    def test_build_www_authenticate_basic(self):
        # Test vector from RFC 7617
        self.assertEqual(
            build_www_authenticate_basic("foo"), 'Basic realm="foo", charset="UTF-8"'
        )

    def test_build_www_authenticate_basic_invalid_realm(self):
        # Realm contains a control character forbidden in quoted-string encoding
        with self.assertRaises(ValueError):
            build_www_authenticate_basic("\u0007")

    def test_build_authorization_basic(self):
        # Test vector from RFC 7617
        self.assertEqual(
            build_authorization_basic("Aladdin", "open sesame"),
            "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==",
        )

    def test_build_authorization_basic_utf8(self):
        # Test vector from RFC 7617
        self.assertEqual(
            build_authorization_basic("test", "123£"), "Basic dGVzdDoxMjPCow=="
        )

    def test_parse_authorization_basic(self):
        for header, parsed in [
            ("Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==", ("Aladdin", "open sesame")),
            # Password contains non-ASCII character
            ("Basic dGVzdDoxMjPCow==", ("test", "123£")),
            # Password contains a colon
            ("Basic YWxhZGRpbjpvcGVuOnNlc2FtZQ==", ("aladdin", "open:sesame")),
            # Scheme name must be case insensitive
            ("basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==", ("Aladdin", "open sesame")),
        ]:
            with self.subTest(header=header):
                self.assertEqual(parse_authorization_basic(header), parsed)

    def test_parse_authorization_basic_invalid_header_format(self):
        for header in [
            "// Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==",
            "Basic\tQWxhZGRpbjpvcGVuIHNlc2FtZQ==",
            "Basic ****************************",
            "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ== //",
        ]:
            with self.subTest(header=header):
                with self.assertRaises(InvalidHeaderFormat):
                    parse_authorization_basic(header)

    def test_parse_authorization_basic_invalid_header_value(self):
        for header in [
            "Digest ...",
            "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ",
            "Basic QWxhZGNlc2FtZQ==",
        ]:
            with self.subTest(header=header):
                with self.assertRaises(InvalidHeaderValue):
                    parse_authorization_basic(header)
