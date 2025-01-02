from urllib.parse import urlsplit, parse_qsl


def connect_received(request_headers, response_headers):
    for data in request_headers:
        if data[0] == b':path':
            path = data[1].decode('utf-8')

            qs = dict(parse_qsl(urlsplit(path).query))
            for key, value in qs.items():
                response_headers.append((key.encode('utf-8'), value.encode('utf-8')))

            break
    return
