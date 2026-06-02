from typing import Optional, Tuple
from urllib.parse import urlsplit, parse_qsl

return_stream_id = 0;
summary : bytes = [];

def session_established(session):
    # When a WebTransport session is established, a bidirectional stream is
    # created by the server, which is used to echo back stream data from the
    # client.
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
    global summary;
    # need an initial value to replace
    session.stash.put(key=token, value=summary)

def stream_data_received(session,
                         stream_id: int,
                         data: bytes,
                         stream_ended: bool):
    # we want to record the order that data arrives, and feed that ordering back to
    # the sender.  Instead of echoing all the data, we'll send back
    # just the first byte of each message.   This requires the sender to send buffers
    # filled with only a single byte value.
    # The test can then examine the stream of data received by the server to
    # determine if orderings are correct.
    # note that the size in bytes received here can vary wildly

    # Send back the data on the control stream
    global summary
    summary += data[0:1]
    token = session.dict_for_handlers['token']
    old_data = session.stash.take(key=token) or {}
    session.stash.put(key=token, value=summary)

def stream_reset(session, stream_id: int, error_code: int) -> None:
    global summary;
    token = session.dict_for_handlers['token']
    session.stash.put(key=token, value=summary)
    summary = []

# do something different to include datagrams...
def datagram_received(session, data: bytes):
    session.send_datagram(data)
