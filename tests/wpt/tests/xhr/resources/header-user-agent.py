def main(request, response):
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    response.headers.set(b"Access-Control-Max-Age", 0)
    response.headers.set(b'Access-Control-Allow-Headers', b"x-test")

    if request.method == u"OPTIONS":
        if not request.headers.get(b"User-Agent"):
            response.content = b"FAIL: User-Agent header missing in preflight request."
            response.status = 400
    else:
        if request.headers.get(b"User-Agent"):
            response.content = b"PASS"
        else:
            response.content = b"FAIL: User-Agent header missing in request"
            response.status = 400
