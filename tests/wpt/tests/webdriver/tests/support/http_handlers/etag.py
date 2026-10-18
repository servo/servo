ETAG = b'"v1"'


def main(request, response):
    """Handler that returns a HTML response with an ETag and
    Cache-Control no-cache. Conditional requests with a matching If-None-Match
    header get a 304 response.
    """
    response.headers.set(b"ETag", ETAG)
    response.headers.set(b"Cache-Control", b"no-cache")

    if request.headers.get(b"If-None-Match") == ETAG:
        response.status = 304
        return ""

    response.headers.set(b"Content-Type", b"text/html")
    response.status = 200
    return "<html><body>etag HTTP Response</body></html>"
