def main(request, response):
    response.headers.set(b"Content-Type", request.GET.first(b"type"))
    origin = request.headers.get('Origin')
    if origin is not None:
        response.headers.set(b"Access-Control-Allow-Origin", origin)
        response.headers.set(b"Access-Control-Allow-Credentials", b"true")

    return request.GET.first(b"content")
