from typing import Optional
from urllib.parse import urlsplit, parse_qsl


def session_established(session):
    path: Optional[bytes] = None
    for key, value in session.request_headers:
        if key == b':path':
            path = value
    assert path is not None
    qs = dict(parse_qsl(urlsplit(path).query))
    code = qs[b'code'] if b'code' in qs else None
    reason = qs[b'reason'] if b'reason' in qs else b''
    close_info = None if code is None else (int(code), reason)

    session.close(close_info)
