# mypy: allow-untyped-defs

from wptserve.request import RequestHeaders

from ..headers import H3Headers


def test_headers_decode_and_normalize_pseudo_headers():
    headers = H3Headers([
        (b':method', b'CONNECT'),
        (b':protocol', b'websocket'),
        (b'sec-websocket-version', b'13'),
        # custom header
        (b'x-value', b'\xff'),
    ])

    assert list(headers.raw_headers.items()) == [
        (':method', 'CONNECT'),
        (':protocol', 'websocket'),
        ('sec-websocket-version', '13'),
        ('x-value', '\xff'),
    ]
    assert headers[':method'] == 'CONNECT'
    assert headers['method'] == 'CONNECT'
    assert headers[':protocol'] == 'websocket'
    assert headers['protocol'] == 'websocket'
    assert headers['x-value'] == '\xff'


def test_split_cookie_fields_are_joined_with_semicolon():
    headers = H3Headers([
        (b'cookie', b'first=one'),
        (b'x-value', b'original'),
        (b'cookie', b'second=two'),
        (b'x-value', b'last'),
    ])

    assert headers.raw_headers['cookie'] == 'first=one; second=two'
    assert headers['cookie'] == 'first=one; second=two'
    assert RequestHeaders(headers).get('cookie') == b'first=one; second=two'
    assert headers.raw_headers['x-value'] == 'last'
