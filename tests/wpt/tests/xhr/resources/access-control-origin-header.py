def main(request, response):
    response.headers.set(b"Content-Type", b"text/plain")
    response.headers.set(b"Cache-Control", b"no-cache, no-store")
    response.headers.set(b"Access-Control-Allow-External", b"true")
    response.headers.set(b"Access-Control-Allow-Origin", b"*")

    response.content = b"PASS: Cross-domain access allowed.\n"
    response.content += b"HTTP_ORIGIN: " + request.headers.get(b"origin")
