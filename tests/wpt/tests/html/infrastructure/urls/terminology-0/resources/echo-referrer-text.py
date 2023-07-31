def main(request, response):
    response_headers = [(b"Content-Type", b"text/plain")]
    body = b"%s"% request.headers.get(b"referer", b"")
    return (200, response_headers, body)
