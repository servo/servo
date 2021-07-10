def main(request, response):
    response.headers.set(b"Content-Type", b"text/event-stream")
    return u""
