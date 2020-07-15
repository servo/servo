def main(request, response):
    response.headers.set(b"Cache-Control", b"no-store")
    response.headers.set(b"Access-Control-Allow-Origin", b"*")

    if request.method == u"OPTIONS":
        if b"origin" in request.headers.get(b"Access-Control-Request-Headers").lower():
            response.status = 400
            response.content = b"Error: 'origin' included in Access-Control-Request-Headers"
        else:
            response.headers.set(b"Access-Control-Allow-Headers", b"x-pass")
    else:
        response.content = request.headers.get(b"x-pass")
