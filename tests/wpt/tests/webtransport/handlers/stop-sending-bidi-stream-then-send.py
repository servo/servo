from typing import Optional
from urllib.parse import urlsplit, parse_qsl


def session_established(session):
    path: Optional[bytes] = None
    for key, value in session.request_headers:
        if key == b':path':
            path = value
    assert path is not None
    qs = dict(parse_qsl(urlsplit(path).query))
    session.dict_for_handlers['code'] = int(qs[b'code'])
    session.dict_for_handlers['stopped'] = False


# Sends STOP_SENDING on a client-created bidirectional stream's first data,
# without resetting the server's send half, then sends data and a FIN on it.
def stream_data_received(session,
                         stream_id: int,
                         data: bytes,
                         stream_ended: bool):
    if session.dict_for_handlers['stopped']:
        return
    session.dict_for_handlers['stopped'] = True
    session.stop_stream(stream_id, session.dict_for_handlers['code'])
    session.send_stream_data(stream_id, b'abc', end_stream=True)
