def main(request, response):
    name = request.GET.first(b"name")
    value = request.GET.first(b"value")
    source_origin = request.headers.get(b"origin", None)

    response_headers = [(b"Set-Cookie", name + b"=" + value),
                        (b"Access-Control-Allow-Origin", source_origin),
                        (b"Access-Control-Allow-Credentials", b"true")]
    return (200, response_headers, u"")
