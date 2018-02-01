# This is a copy of fetch/api/resources/echo-content.py since it's more
# convenient in this directory due to service worker's path restriction.
def main(request, response):

    headers = [("X-Request-Method", request.method),
               ("X-Request-Content-Length", request.headers.get("Content-Length", "NO")),
               ("X-Request-Content-Type", request.headers.get("Content-Type", "NO"))]

    content = request.body

    return headers, content
