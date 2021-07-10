def main(request, response):
    headers = [(b"Location", b"")]
    return 302, headers, b""
