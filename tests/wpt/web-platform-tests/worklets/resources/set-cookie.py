def main(request, response):
    name = request.GET.first("name")
    source_origin = request.headers.get("origin", None);
    response.headers.set("Set-Cookie", name + "=value")
    response.headers.set("Access-Control-Allow-Origin", source_origin)
    response.headers.set("Access-Control-Allow-Credentials", "true")
