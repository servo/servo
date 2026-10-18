import binascii

def main(request, response):
    headers = [(b"content-type", b"text/plain")]
    body = []
    for header in request.GET.first(b"headers").split(b"|"):
        body.append(header + b": " + binascii.hexlify(request.headers.get(header, b"")))
    return headers, b"\n".join(body)
