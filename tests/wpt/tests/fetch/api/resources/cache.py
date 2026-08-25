ETAG = b'"123abc"'
CONTENT_TYPE = b"text/plain"
CONTENT = b"lorem ipsum dolor sit amet"


def main(request, response):
    # let caching kick in if possible (conditional GET)
    etag = request.headers.get(b"If-None-Match", None)
    if etag == ETAG:
        response.headers.set(b"X-HTTP-STATUS", 304)
        response.status = (304, b"Not Modified")
        return b""

    # cache miss, so respond with the actual content
    response.status = (200, b"OK")
    response.headers.set(b"ETag", ETAG)
    response.headers.set(b"Content-Type", CONTENT_TYPE)
    return CONTENT
