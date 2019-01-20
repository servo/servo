def main(request, response):
    response.status = 302
    location = request.GET.first("location")
    response.headers.set("Location", location)
    response.headers.set("Access-Control-Allow-Origin", "*")
