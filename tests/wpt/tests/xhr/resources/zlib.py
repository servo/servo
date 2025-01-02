import zlib

def main(request, response):
    if b"content" in request.GET:
        output = request.GET[b"content"]
    else:
        output = request.body

    output = zlib.compress(output, 9)

    headers = [(b"Content-type", b"text/plain"),
               (b"Content-Encoding", b"deflate"),
               (b"X-Request-Method", request.method),
               (b"X-Request-Query", request.url_parts.query if request.url_parts.query else b"NO"),
               (b"X-Request-Content-Length", request.headers.get(b"Content-Length", b"NO")),
               (b"X-Request-Content-Type", request.headers.get(b"Content-Type", b"NO")),
               (b"Content-Length", len(output))]

    return headers, output
