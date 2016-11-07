ETAG = '"123abc"'
CONTENT_TYPE = "text/plain"
CONTENT = "lorem ipsum dolor sit amet"


def main(request, response):
    # let caching kick in if possible (conditional GET)
    etag = request.headers.get("If-None-Match", None)
    if etag == ETAG:
        response.headers.set("X-HTTP-STATUS", 304)
        response.status = (304, "Not Modified")
        return ""

    # cache miss, so respond with the actual content
    response.status = (200, "OK")
    response.headers.set("ETag", ETAG)
    response.headers.set("Content-Type", CONTENT_TYPE)
    return CONTENT
