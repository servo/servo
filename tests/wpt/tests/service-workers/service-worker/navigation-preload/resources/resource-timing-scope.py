import zlib

def main(request, response):
    type = request.GET.first(b"type")

    if type == "normal":
        content = b"This is Navigation Preload Resource Timing test."
        output = zlib.compress(content, 9)
        headers = [(b"Content-type", b"text/plain"),
                   (b"Content-Encoding", b"deflate"),
                   (b"X-Decoded-Body-Size", len(content)),
                   (b"X-Encoded-Body-Size", len(output)),
                   (b"Content-Length", len(output))]
        return headers, output

    if type == b"redirect":
        response.status = 302
        response.headers.append(b"Location", b"redirect-redirected.html")
        return b""
