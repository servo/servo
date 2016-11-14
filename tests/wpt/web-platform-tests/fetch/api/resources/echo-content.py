def main(request, response):

    headers = [("X-Request-Method", request.method),
               ("X-Request-Content-Length", request.headers.get("Content-Length", "NO")),
               ("X-Request-Content-Type", request.headers.get("Content-Type", "NO"))]

    content = request.body

    return headers, content
