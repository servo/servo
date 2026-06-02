def main(request, response):
    if request.headers.get(b"Origin") is not None:
        response.headers.set(
            b"Access-Control-Allow-Origin", request.headers.get(b"Origin")
        )
    return ""
