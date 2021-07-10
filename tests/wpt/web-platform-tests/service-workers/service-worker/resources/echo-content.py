# This is a copy of fetch/api/resources/echo-content.py since it's more
# convenient in this directory due to service worker's path restriction.
from wptserve.utils import isomorphic_encode

def main(request, response):

    headers = [(b"X-Request-Method", isomorphic_encode(request.method)),
               (b"X-Request-Content-Length", request.headers.get(b"Content-Length", b"NO")),
               (b"X-Request-Content-Type", request.headers.get(b"Content-Type", b"NO")),

               # Avoid any kind of content sniffing on the response.
               (b"Content-Type", b"text/plain")]

    content = request.body

    return headers, content
