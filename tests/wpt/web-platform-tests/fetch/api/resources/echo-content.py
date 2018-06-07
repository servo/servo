def main(request, response):

    headers = [("X-Request-Method", request.method),
               ("X-Request-Content-Length", request.headers.get("Content-Length", "NO")),
               ("X-Request-Content-Type", request.headers.get("Content-Type", "NO")),
               # Avoid any kind of content sniffing on the response.
               ("Content-Type", "text/plain")]
    content = request.body

    return headers, content
