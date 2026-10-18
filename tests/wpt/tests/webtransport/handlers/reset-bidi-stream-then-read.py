from typing import Optional
from urllib.parse import urlsplit, parse_qsl


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
    session.dict_for_handlers['token'] = token
    session.dict_for_handlers['code'] = int(qs[b'code'])
    session.dict_for_handlers['received'] = 0


# Resets the send half of a client-created bidirectional stream on its first
# data, without sending STOP_SENDING, then keeps counting what the client
# writes to the still-open receive half.
def stream_data_received(session,
                         stream_id: int,
                         data: bytes,
                         stream_ended: bool):
    if session.dict_for_handlers['received'] == 0:
        session.reset_stream(stream_id, session.dict_for_handlers['code'])
    session.dict_for_handlers['received'] += len(data)

    if stream_ended:
        token = session.dict_for_handlers['token']
        stashed_data = session.stash.take(key=token) or {}
        stashed_data['stream-close-info'] = {
            'source': 'FIN',
            'received': session.dict_for_handlers['received']
        }
        session.stash.put(key=token, value=stashed_data)


def stream_reset(session, stream_id: int, error_code: int) -> None:
    token = session.dict_for_handlers['token']
    stashed_data = session.stash.take(key=token) or {}
    stashed_data['stream-close-info'] = {'source': 'reset', 'code': error_code}
    session.stash.put(key=token, value=stashed_data)
