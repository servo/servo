import json
from urllib.parse import urlsplit, parse_qsl


def session_established(session):
    path = ''
    for name, value in session.request_headers:
        if name == b':path':
            path = value.decode('utf-8')
            break

    query = dict(parse_qsl(urlsplit(path).query))
    as_list = query.get('format') == 'list'

    headers = {}
    for name, value in session.request_headers:
        name = name.decode('utf-8')
        value = value.decode('utf-8')
        if as_list:
            headers.setdefault(name, []).append(value)
        else:
            headers[name] = value

    stream_id = session.create_unidirectional_stream()
    data = json.dumps(headers).encode('utf-8')
    session.send_stream_data(stream_id, data, end_stream=True)
