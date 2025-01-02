def main(request, response):
    response.headers.set(b"Cache-Control", b"no-store")
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    response.headers.set(b"Access-Control-Max-Age", 0)

    if request.method == u"OPTIONS":
        if b"x-test" in [header.strip(b" ") for header in
                         request.headers.get(b"Access-Control-Request-Headers").split(b",")]:
            response.headers.set(b"Access-Control-Allow-Headers", b"X-Test")
        else:
            response.status = 400
    elif request.method == u"GET":
        if request.headers.get(b"X-Test"):
            response.content = b"PASS"
        else:
            response.status = 400
