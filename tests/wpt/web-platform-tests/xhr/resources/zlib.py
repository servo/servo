import zlib

def main(request, response):
    if "content" in request.GET:
        output = request.GET["content"]
    else:
        output = request.body

    output = zlib.compress(output, 9)

    headers = [("Content-type", "text/plain"),
               ("Content-Encoding", "deflate"),
               ("X-Request-Method", request.method),
               ("X-Request-Query", request.url_parts.query if request.url_parts.query else "NO"),
               ("X-Request-Content-Length", request.headers.get("Content-Length", "NO")),
               ("X-Request-Content-Type", request.headers.get("Content-Type", "NO")),
               ("Content-Length", len(output))]

    return headers, output
