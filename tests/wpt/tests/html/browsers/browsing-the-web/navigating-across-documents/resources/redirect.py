def main(request, response):
    location = request.GET.first(b"location")
    response.status = 302
    response.headers.set(b"Location", location)
