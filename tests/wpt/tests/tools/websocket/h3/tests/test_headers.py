# mypy: allow-untyped-defs

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
