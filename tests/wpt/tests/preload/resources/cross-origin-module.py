def main(request, response):
    headers = [
        (b"Content-Type", b"text/javascript"),
        (b"Access-Control-Allow-Origin", request.headers.get(b"Origin")),
        (b"Timing-Allow-Origin", request.headers.get(b"Origin")),
        (b"Access-Control-Allow-Credentials", b"true")
    ]

    return headers, u"// Cross-origin module, nothing to see here"
