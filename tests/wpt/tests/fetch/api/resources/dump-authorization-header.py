def main(request, response):
    headers = [(b"Content-Type", "text/html"),
               (b"Cache-Control", b"no-cache")]

    if b"Origin" in request.headers:
        headers.append((b"Access-Control-Allow-Origin", request.headers.get(b"Origin", b"")))
        headers.append((b"Access-Control-Allow-Credentials", b"true"))
    else:
        headers.append((b"Access-Control-Allow-Origin", b"*"))
    headers.append((b"Access-Control-Allow-Headers", b'Authorization'))

    if b"authorization" in request.headers:
        return 200, headers, request.headers.get(b"Authorization")
    return 200, headers, "none"
