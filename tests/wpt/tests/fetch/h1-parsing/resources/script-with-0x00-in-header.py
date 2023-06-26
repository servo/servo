def main(request, response):
    response.headers.set(b"Content-Type", b"text/javascript")
    response.headers.set(b"Custom", b"\0")
    return b"var thisIsJavaScript = 0"
