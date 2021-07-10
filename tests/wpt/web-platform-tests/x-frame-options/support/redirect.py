def main(request, response):
    response.status = 302
    response.headers.set(b"X-Frame-Options", request.GET.first(b"value"))
    response.headers.set(b"Location", request.GET.first(b"url"))
