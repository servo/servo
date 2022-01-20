def main(request, response):
    response.headers.set(b"Content-Type", b"text/plain")
    response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))

    response.content = b"PASS: Cross-domain access allowed."
