def main(request, response):
    name = request.GET.first("name")
    value = request.GET.first("value")
    source_origin = request.headers.get("origin", None)

    response_headers = [("Set-Cookie", name + "=" + value),
                        ("Access-Control-Allow-Origin", source_origin),
                        ("Access-Control-Allow-Credentials", "true")]
    return (200, response_headers, "")
