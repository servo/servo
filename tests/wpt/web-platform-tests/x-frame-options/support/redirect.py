def main(request, response):
    response.status = 302
    response.headers.set("X-Frame-Options", request.GET.first("value"))
    response.headers.set("Location", request.GET.first("url"))
