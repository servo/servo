def main(request, response):
    response.status = 302
    location = request.GET.first(b"location")
    response.headers.set(b"Location", location)
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
