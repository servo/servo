import zlib

def main(request, response):
    type = request.GET.first("type")

    if type == "normal":
        content = "This is Navigation Preload Resource Timing test."
        output = zlib.compress(content, 9)
        headers = [("Content-type", "text/plain"),
                   ("Content-Encoding", "deflate"),
                   ("X-Decoded-Body-Size", len(content)),
                   ("X-Encoded-Body-Size", len(output)),
                   ("Content-Length", len(output))]
        return headers, output

    if type == "redirect":
        response.status = 302
        response.headers.append("Location", "redirect-redirected.html")
        return ""
