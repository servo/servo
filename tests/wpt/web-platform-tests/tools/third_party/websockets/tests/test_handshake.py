import contextlib
import unittest

from websockets.exceptions import (
    InvalidHandshake,
    InvalidHeader,
    InvalidHeaderValue,
    InvalidUpgrade,
)
from websockets.handshake import *
from websockets.handshake import accept  # private API
from websockets.http import Headers


class HandshakeTests(unittest.TestCase):
    def test_accept(self):
        # Test vector from RFC 6455
        key = "dGhlIHNhbXBsZSBub25jZQ=="
        acc = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
        self.assertEqual(accept(key), acc)

    def test_round_trip(self):
        request_headers = Headers()
        request_key = build_request(request_headers)
        response_key = check_request(request_headers)
        self.assertEqual(request_key, response_key)
        response_headers = Headers()
        build_response(response_headers, response_key)
        check_response(response_headers, request_key)

    @contextlib.contextmanager
    def assertValidRequestHeaders(self):
        """
        Provide request headers for modification.

        Assert that the transformation kept them valid.

        """
        headers = Headers()
        build_request(headers)
        yield headers
        check_request(headers)

    @contextlib.contextmanager
    def assertInvalidRequestHeaders(self, exc_type):
        """
        Provide request headers for modification.

        Assert that the transformation made them invalid.

        """
        headers = Headers()
        build_request(headers)
        yield headers
        assert issubclass(exc_type, InvalidHandshake)
        with self.assertRaises(exc_type):
            check_request(headers)

    def test_request_invalid_connection(self):
        with self.assertInvalidRequestHeaders(InvalidUpgrade) as headers:
            del headers["Connection"]
            headers["Connection"] = "Downgrade"

    def test_request_missing_connection(self):
        with self.assertInvalidRequestHeaders(InvalidUpgrade) as headers:
            del headers["Connection"]

    def test_request_additional_connection(self):
        with self.assertValidRequestHeaders() as headers:
            headers["Connection"] = "close"

    def test_request_invalid_upgrade(self):
        with self.assertInvalidRequestHeaders(InvalidUpgrade) as headers:
            del headers["Upgrade"]
            headers["Upgrade"] = "socketweb"

    def test_request_missing_upgrade(self):
        with self.assertInvalidRequestHeaders(InvalidUpgrade) as headers:
            del headers["Upgrade"]

    def test_request_additional_upgrade(self):
        with self.assertInvalidRequestHeaders(InvalidUpgrade) as headers:
            headers["Upgrade"] = "socketweb"

    def test_request_invalid_key_not_base64(self):
        with self.assertInvalidRequestHeaders(InvalidHeaderValue) as headers:
            del headers["Sec-WebSocket-Key"]
            headers["Sec-WebSocket-Key"] = "!@#$%^&*()"

    def test_request_invalid_key_not_well_padded(self):
        with self.assertInvalidRequestHeaders(InvalidHeaderValue) as headers:
            del headers["Sec-WebSocket-Key"]
            headers["Sec-WebSocket-Key"] = "CSIRmL8dWYxeAdr/XpEHRw"

    def test_request_invalid_key_not_16_bytes_long(self):
        with self.assertInvalidRequestHeaders(InvalidHeaderValue) as headers:
            del headers["Sec-WebSocket-Key"]
            headers["Sec-WebSocket-Key"] = "ZLpprpvK4PE="

    def test_request_missing_key(self):
        with self.assertInvalidRequestHeaders(InvalidHeader) as headers:
            del headers["Sec-WebSocket-Key"]

    def test_request_additional_key(self):
        with self.assertInvalidRequestHeaders(InvalidHeader) as headers:
            # This duplicates the Sec-WebSocket-Key header.
            headers["Sec-WebSocket-Key"] = headers["Sec-WebSocket-Key"]

    def test_request_invalid_version(self):
        with self.assertInvalidRequestHeaders(InvalidHeaderValue) as headers:
            del headers["Sec-WebSocket-Version"]
            headers["Sec-WebSocket-Version"] = "42"

    def test_request_missing_version(self):
        with self.assertInvalidRequestHeaders(InvalidHeader) as headers:
            del headers["Sec-WebSocket-Version"]

    def test_request_additional_version(self):
        with self.assertInvalidRequestHeaders(InvalidHeader) as headers:
            # This duplicates the Sec-WebSocket-Version header.
            headers["Sec-WebSocket-Version"] = headers["Sec-WebSocket-Version"]

    @contextlib.contextmanager
    def assertValidResponseHeaders(self, key="CSIRmL8dWYxeAdr/XpEHRw=="):
        """
        Provide response headers for modification.

        Assert that the transformation kept them valid.

        """
        headers = Headers()
        build_response(headers, key)
        yield headers
        check_response(headers, key)

    @contextlib.contextmanager
    def assertInvalidResponseHeaders(self, exc_type, key="CSIRmL8dWYxeAdr/XpEHRw=="):
        """
        Provide response headers for modification.

        Assert that the transformation made them invalid.

        """
        headers = Headers()
        build_response(headers, key)
        yield headers
        assert issubclass(exc_type, InvalidHandshake)
        with self.assertRaises(exc_type):
            check_response(headers, key)

    def test_response_invalid_connection(self):
        with self.assertInvalidResponseHeaders(InvalidUpgrade) as headers:
            del headers["Connection"]
            headers["Connection"] = "Downgrade"

    def test_response_missing_connection(self):
        with self.assertInvalidResponseHeaders(InvalidUpgrade) as headers:
            del headers["Connection"]

    def test_response_additional_connection(self):
        with self.assertValidResponseHeaders() as headers:
            headers["Connection"] = "close"

    def test_response_invalid_upgrade(self):
        with self.assertInvalidResponseHeaders(InvalidUpgrade) as headers:
            del headers["Upgrade"]
            headers["Upgrade"] = "socketweb"

    def test_response_missing_upgrade(self):
        with self.assertInvalidResponseHeaders(InvalidUpgrade) as headers:
            del headers["Upgrade"]

    def test_response_additional_upgrade(self):
        with self.assertInvalidResponseHeaders(InvalidUpgrade) as headers:
            headers["Upgrade"] = "socketweb"

    def test_response_invalid_accept(self):
        with self.assertInvalidResponseHeaders(InvalidHeaderValue) as headers:
            del headers["Sec-WebSocket-Accept"]
            other_key = "1Eq4UDEFQYg3YspNgqxv5g=="
            headers["Sec-WebSocket-Accept"] = accept(other_key)

    def test_response_missing_accept(self):
        with self.assertInvalidResponseHeaders(InvalidHeader) as headers:
            del headers["Sec-WebSocket-Accept"]

    def test_response_additional_accept(self):
        with self.assertInvalidResponseHeaders(InvalidHeader) as headers:
            # This duplicates the Sec-WebSocket-Accept header.
            headers["Sec-WebSocket-Accept"] = headers["Sec-WebSocket-Accept"]
