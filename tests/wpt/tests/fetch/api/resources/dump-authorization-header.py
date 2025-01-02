def main(request, response):
    headers = [(b"Content-Type", "text/html"),
               (b"Cache-Control", b"no-cache")]

    if (request.GET.first(b"strip_auth_header", False) and request.method == "OPTIONS" and
        b"authorization" in request.headers.get(b"Access-Control-Request-Headers", b"").lower()):
        # Auth header should not be sent for preflight after cross-origin redirect.
        return 500, headers, "fail"

    if b"Origin" in request.headers:
        headers.append((b"Access-Control-Allow-Origin", request.headers.get(b"Origin", b"")))
        headers.append((b"Access-Control-Allow-Credentials", b"true"))
    else:
        headers.append((b"Access-Control-Allow-Origin", b"*"))
    headers.append((b"Access-Control-Allow-Headers", b'Authorization'))

    if b"authorization" in request.headers:
        return 200, headers, request.headers.get(b"Authorization")
    return 200, headers, "none"
