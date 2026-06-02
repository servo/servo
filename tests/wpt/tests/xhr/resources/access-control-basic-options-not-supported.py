def main(request, response):
    response.headers.set(b"Cache-Control", b"no-store")

    # Allow simple requests, but deny preflight
    if request.method != u"OPTIONS":
        if b"origin" in request.headers:
            response.headers.set(b"Access-Control-Allow-Credentials", b"true")
            response.headers.set(b"Access-Control-Allow-Origin", request.headers[b"origin"])
        else:
            response.status = 500
    else:
        response.status = 400
