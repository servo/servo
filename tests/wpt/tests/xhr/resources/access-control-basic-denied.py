def main(request, response):
    response.headers.set(b"Cache-Control", b"no-store")
    response.headers.set(b"Content-Type", b"text/plain")

    response.text = b"FAIL: Cross-domain access allowed."
