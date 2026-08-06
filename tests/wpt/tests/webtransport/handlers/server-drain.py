from typing import Optional
from urllib.parse import urlsplit, parse_qsl


def session_established(session):
    path: Optional[bytes] = None
    for key, value in session.request_headers:
        if key == b':path':
            path = value
    assert path is not None

    path_str = path.decode('utf-8') if isinstance(path, bytes) else path
    parsed = urlsplit(path_str)

    # Check if 'drain' appears anywhere in the query string
    if 'drain' in parsed.query:
        session.initiate_draining()
