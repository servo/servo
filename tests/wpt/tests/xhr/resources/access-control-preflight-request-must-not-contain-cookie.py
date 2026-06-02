def main(request, response):
    if request.method == u"OPTIONS" and request.cookies.get(b"foo"):
        response.status = 400
    else:
        response.headers.set(b"Cache-Control", b"no-store")
        response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))
        response.headers.set(b"Access-Control-Allow-Credentials", b"true")
        response.headers.set(b"Access-Control-Allow-Headers", b"X-Proprietary-Header")
        response.headers.set(b"Connection", b"close")

        if request.cookies.get(b"foo"):
            response.content = request.cookies[b"foo"].value
