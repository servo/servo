# mypy: allow-untyped-defs

"""This file provides the opening handshake processor for Bootstrapping
WebSockets with HTTP/3 protocol (RFC 9220).

Specification:
https://datatracker.ietf.org/doc/html/rfc9220
"""

from pywebsocket3 import common
from pywebsocket3.handshake.base import get_mandatory_header
from pywebsocket3.handshake.base import HandshakeException
from pywebsocket3.handshake.base import HandshakerBase
from pywebsocket3.handshake.base import validate_mandatory_header


def check_connect_method(request):
    if request.method != 'CONNECT':
        raise HandshakeException('Method is not CONNECT: %r' % request.method)


class WsH3Handshaker(HandshakerBase):
    """Bootstrapping handshake processor for WebSocket over HTTP/3."""

    def __init__(self, request, dispatcher):
        super().__init__(request, dispatcher)

    def _transform_header(self, header):
        return header.lower()

    def _protocol_rfc(self):
        return 'RFC 9220'

    def _validate_request(self):
        check_connect_method(self._request)
        validate_mandatory_header(self._request, ':protocol', 'websocket')
        get_mandatory_header(self._request, 'authority')

    def _set_accept(self):
        # irrelevant for HTTP/3 handshake
        pass

    def _send_handshake(self):
        # The caller sends the HTTP/3 HEADERS frame after do_handshake()
        # returns.
        self._request.status = 200

        self._request.headers_out['upgrade'] = common.WEBSOCKET_UPGRADE_TYPE
        self._request.headers_out[
            'connection'] = common.UPGRADE_CONNECTION_TYPE

        if self._request.ws_protocol is not None:
            self._request.headers_out[
                'sec-websocket-protocol'] = self._request.ws_protocol

        if (self._request.ws_extensions is not None and
                len(self._request.ws_extensions) != 0):
            self._request.headers_out[
                'sec-websocket-extensions'] = common.format_extensions(
                    self._request.ws_extensions)

        for name, value in self._request.extra_headers:
            self._request.headers_out[name] = value
