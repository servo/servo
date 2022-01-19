def main(request, response):
    location = request.GET.first(b"location")
    response.status = 302
    response.headers.set(b"Location", location)

    if b"allow_origin" in request.GET:
        response.headers.set(b"Access-Control-Allow-Origin", request.GET.first(b"allow_origin"))

    if b"timing_allow_origin" in request.GET:
        response.headers.set(b"Timing-Allow-Origin", request.GET.first(b"timing_allow_origin"))
