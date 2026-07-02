# mypy: allow-untyped-defs

from types import SimpleNamespace

import pytest

from pywebsocket3 import common
from pywebsocket3.handshake.base import HandshakeException

from ..headers import H3Headers
from ..ws_h3_handshake import WsH3Handshaker


class _FakeDispatcher:
    def __init__(self, protocol=None, extra_headers=()):
        self.protocol = protocol
        self.extra_headers = extra_headers
        self.called = False

    def do_extra_handshake(self, request):
        self.called = True
        if self.protocol is not None:
            request.ws_protocol = self.protocol
        request.extra_headers.extend(self.extra_headers)


def _make_request(headers=(), method='CONNECT'):
    raw_headers = [
        (b':method', b'CONNECT'),
        (b':protocol', b'websocket'),
        (b':authority', b'web-platform.test:11001'),
        (b':path', b'/echo'),
        (b'sec-websocket-version', b'13'),
    ]
    raw_headers.extend(headers)
    return SimpleNamespace(
        method=method,
        uri='/echo',
        headers_in=H3Headers(raw_headers),
        headers_out={},
        status=None,
        connection=None,
    )


# Valid H3 WebSocket CONNECT requests prepare the response headers.
def test_do_handshake_prepares_h3_response_headers():
    request = _make_request([
        (b'sec-websocket-protocol', b'chat'),
    ])
    dispatcher = _FakeDispatcher(
        protocol='chat',
        extra_headers=[('x-extra', 'value')],
    )

    WsH3Handshaker(request, dispatcher).do_handshake()

    assert dispatcher.called
    assert request.status == 200
    assert request.headers_out['upgrade'] == common.WEBSOCKET_UPGRADE_TYPE
    assert request.headers_out['connection'] == common.UPGRADE_CONNECTION_TYPE
    assert request.headers_out['sec-websocket-protocol'] == 'chat'
    assert request.headers_out['x-extra'] == 'value'
    assert 'sec-websocket-accept' not in request.headers_out
    assert request.ws_resource == '/echo'
    assert request.ws_version == common.VERSION_HYBI_LATEST


# H3 WebSocket handshakes require the CONNECT method.
def test_do_handshake_rejects_non_connect_method():
    request = _make_request(method='GET')

    with pytest.raises(HandshakeException, match='Method is not CONNECT'):
        WsH3Handshaker(request, _FakeDispatcher()).do_handshake()


# H3 WebSocket handshakes require the :protocol header to be websocket.
def test_do_handshake_requires_websocket_protocol():
    request = _make_request()
    request.headers_in[':protocol'] = 'connect-udp'

    with pytest.raises(HandshakeException, match='Expected .*websocket'):
        WsH3Handshaker(request, _FakeDispatcher()).do_handshake()


# H3 WebSocket handshakes require the authority header.
def test_do_handshake_requires_authority():
    request = _make_request()
    del request.headers_in['authority']

    with pytest.raises(HandshakeException,
                       match='Header authority is not defined'):
        WsH3Handshaker(request, _FakeDispatcher()).do_handshake()


# Accepted WebSocket extensions are formatted in the response headers.
def test_send_handshake_formats_extensions():
    extension = common.ExtensionParameter('permessage-deflate')
    extension.add_parameter('server_no_context_takeover', None)
    request = _make_request()
    request.ws_protocol = None
    request.ws_extensions = [extension]
    request.extra_headers = []

    WsH3Handshaker(request, _FakeDispatcher())._send_handshake()

    assert request.status == 200
    assert request.headers_out[
        'sec-websocket-extensions'] == (
            'permessage-deflate; server_no_context_takeover')
