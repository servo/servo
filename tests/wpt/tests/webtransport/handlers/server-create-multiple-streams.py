from urllib.parse import urlsplit, parse_qsl


def session_established(session):
    path = None
    for key, value in session.request_headers:
        if key == b':path':
            path = value
    assert path is not None
    qs = dict(parse_qsl(urlsplit(path).query))
    stream_type = qs.get(b'type', b'bidi').decode()
    count = int(qs.get(b'count', b'3'))
    for i in range(count):
        if stream_type == 'unidi':
            stream_id = session.create_unidirectional_stream()
        else:
            stream_id = session.create_bidirectional_stream()
        session.send_stream_data(stream_id, f'stream{i}'.encode(),
                                 end_stream=True)
