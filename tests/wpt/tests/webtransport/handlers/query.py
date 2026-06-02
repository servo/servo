from typing import Optional
from urllib.parse import urlsplit, parse_qsl
import json


def session_established(session):
    path: Optional[bytes] = None
    for key, value in session.request_headers:
        if key == b':path':
            path = value
    assert path is not None
    qs = dict(parse_qsl(urlsplit(path).query))
    token = qs[b'token']
    if token is None:
        raise Exception('token is missing, path = {}'.format(path))

    stream_id = session.create_unidirectional_stream()
    data = json.dumps(session.stash.take(key=token) or {}).encode('utf-8')
    session.send_stream_data(stream_id, data, end_stream=True)
