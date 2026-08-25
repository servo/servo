def main(request, response):
    accept = request.headers.get(b"accept", b"")
    response_headers = [(b"Content-Type", b"text/plain"),
                        (b"Access-Control-Allow-Origin", b"*")]
    return (200, response_headers, accept)
