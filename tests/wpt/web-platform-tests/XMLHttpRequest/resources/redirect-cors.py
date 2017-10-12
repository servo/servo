def main(request, response):
    location = request.GET.first("location")

    if request.method == "OPTIONS":
        if "redirect_preflight" in request.GET:
            response.status = 302
            response.headers.set("Location", location)
        else:
            response.status = 200
        response.headers.set("Access-Control-Allow-Methods", "GET")
        response.headers.set("Access-Control-Max-Age", 1)
    elif request.method == "GET":
        response.status = 302
        response.headers.set("Location", location)

    if "allow_origin" in request.GET:
        response.headers.set("Access-Control-Allow-Origin", request.headers.get("origin"))

    if "allow_header" in request.GET:
        response.headers.set("Access-Control-Allow-Headers", request.GET.first("allow_header"))
