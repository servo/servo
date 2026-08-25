def main(request, response):
    referrer = request.headers.get(b"referer", b"")
    response_headers = [(b"Content-Type", b"application/json"),
                        (b"Access-Control-Allow-Origin", b"*")]
    return (200, response_headers,
            b'{"referrer": "' + referrer + b'"}')
