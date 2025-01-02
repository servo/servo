def main(request, response):
    return [(b"Content-Type", b"text/plain")], request.headers.get(b"accept", b"NO")
