def main(request, response):
    location = request.GET.first(b"location")

    if request.method == u"OPTIONS":
        if b"redirect_preflight" in request.GET:
            response.status = 302
            response.headers.set(b"Location", location)
        else:
            response.status = 200
        response.headers.set(b"Access-Control-Allow-Methods", b"GET")
        response.headers.set(b"Access-Control-Max-Age", 1)
    elif request.method == u"GET":
        response.status = 302
        response.headers.set(b"Location", location)

    if b"allow_origin" in request.GET:
        response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))

    if b"allow_header" in request.GET:
        response.headers.set(b"Access-Control-Allow-Headers", request.GET.first(b"allow_header"))
