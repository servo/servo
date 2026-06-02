from typing import Optional, Tuple
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
    session.create_bidirectional_stream()


def stream_reset(session, stream_id: int, error_code: int) -> None:
    token = session.dict_for_handlers['token']
    data = session.stash.take(key=token) or {}

    data['stream-close-info'] = {
        'source': 'reset',
        'code': error_code
    }
    session.stash.put(key=token, value=data)


def stream_data_received(session,
                         stream_id: int,
                         data: bytes,
                         stream_ended: bool):
    if stream_ended:
        token = session.dict_for_handlers['token']
        stashed_data = session.stash.take(key=token) or {}
        stashed_data['stream-close-info'] = {
            'source': 'FIN',
        }
        session.stash.put(key=token, value=stashed_data)


def session_closed(
        session, close_info: Optional[Tuple[int, bytes]], abruptly: bool) -> None:
    token = session.dict_for_handlers['token']
    data = session.stash.take(key=token) or {}

    decoded_close_info: Optional[Dict[str, Any]] = None
    if close_info:
        decoded_close_info = {
            'code': close_info[0],
            'reason': close_info[1].decode()
        }

    data['session-close-info'] = {
        'abruptly': abruptly,
        'close_info': decoded_close_info
    }
    session.stash.put(key=token, value=data)
