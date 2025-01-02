def main(request, response):
    try:
        code = int(request.GET.first(b"code", None))
    except:
        code = None

    if request.method == u"OPTIONS":
        if code:
            response.status = code
        response.headers.set(b"Access-Control-Max-Age", 1)
        response.headers.set(b"Access-Control-Allow-Headers", b"x-pass")
    else:
        response.status = 200

    response.headers.set(b"Cache-Control", b"no-store")
    response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))
