from typing import Optional
from urllib.parse import urlsplit, parse_qsl


def session_established(session):
    path: Optional[bytes] = None
    for key, value in session.request_headers:
        if key == b':path':
            path = value
    assert path is not None
    qs = dict(parse_qsl(urlsplit(path).query))
    code = qs[b'code']
    if code is None:
        raise Exception('code is missing, path = {}'.format(path))
    session.dict_for_handlers['code'] = int(code)


def stream_data_received(session,
                         stream_id: int,
                         data: bytes,
                         stream_ended: bool):
    code: int = session.dict_for_handlers['code']
    if session.stream_is_unidirectional(stream_id):
        session.stop_stream(stream_id, code)
    else:
        session.stop_stream(stream_id, code)
        session.reset_stream(stream_id, code)