import gzip as gzip_module
from cStringIO import StringIO

def main(request, response):
    if "content" in request.GET:
        output = request.GET["content"]
    else:
        output = request.body

    out = StringIO()
    with gzip_module.GzipFile(fileobj=out, mode="w") as f:
        f.write(output)
    output = out.getvalue()

    headers = [("Content-type", "text/plain"),
               ("Content-Encoding", "gzip"),
               ("X-Request-Method", request.method),
               ("X-Request-Query", request.url_parts.query if request.url_parts.query else "NO"),
               ("X-Request-Content-Length", request.headers.get("Content-Length", "NO")),
               ("X-Request-Content-Type", request.headers.get("Content-Type", "NO")),
               ("Content-Length", len(output))]

    return headers, output
