import json


def session_established(session):
    headers = {}
    for name, value in session.request_headers:
        headers[name.decode('utf-8')] = value.decode('utf-8')

    stream_id = session.create_unidirectional_stream()
    data = json.dumps(headers).encode('utf-8')
    session.send_stream_data(stream_id, data, end_stream=True)
