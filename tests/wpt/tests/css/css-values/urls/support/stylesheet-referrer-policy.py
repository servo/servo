def main(request, response):
    expected_referrer = request.GET[b'expected_referrer']
    actual_referrer = request.headers.get(b'referer', b'')

    if expected_referrer == b'none':
        match = actual_referrer == b''
    elif expected_referrer == b'origin':
        origin = request.GET[b'origin']
        match = actual_referrer == origin
    elif expected_referrer == b'url':
        url = request.GET[b'url']
        match = actual_referrer == url
    else:
        match = False

    response.add_required_headers = False
    response.writer.write_status(200)
    response.writer.write_header(b"access-control-allow-origin", b"*")
    response.writer.write_header(b"content-type", b"text/css")
    response.writer.write_header(b"cache-control", b"no-cache; must-revalidate")

    if match:
        body = b"#test { color: green; }"
    else:
        body = b"#test { color: red; }"

    response.writer.write_header(b"content-length", len(body))
    response.writer.end_headers()
    response.writer.write(body)
