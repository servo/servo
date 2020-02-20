def main(request, response):
    location = request.GET.first("location")
    response.status = 302
    response.headers.set("Location", location)
