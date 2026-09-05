def main(request, response):
    headers = [(b"Content-Type", b"text/plain"),
               (b"Cache-Control", b"no-store"),
               (b"Access-Control-Allow-Origin", b"*")]
    return headers, request.headers.get(b"cookie", b"")
