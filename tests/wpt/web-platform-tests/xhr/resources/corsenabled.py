import time

from wptserve.utils import isomorphic_encode

def main(request, response):
    headers = [(b"Access-Control-Allow-Origin", b"*"),
               (b"Access-Control-Allow-Credentials", b"true"),
               (b"Access-Control-Allow-Methods", b"GET, POST, PUT, FOO"),
               (b"Access-Control-Allow-Headers", b"x-test, x-foo"),
               (b"Access-Control-Expose-Headers", b"x-request-method, x-request-content-type, x-request-query, x-request-content-length, x-request-data")]

    if b"delay" in request.GET:
        delay = int(request.GET.first(b"delay"))
        time.sleep(delay)

    if b"safelist_content_type" in request.GET:
        headers.append((b"Access-Control-Allow-Headers", b"content-type"))

    headers.append((b"X-Request-Method", isomorphic_encode(request.method)))
    headers.append((b"X-Request-Query", isomorphic_encode(request.url_parts.query) if request.url_parts.query else b"NO"))
    headers.append((b"X-Request-Content-Length", request.headers.get(b"Content-Length", b"NO")))
    headers.append((b"X-Request-Content-Type", request.headers.get(b"Content-Type", b"NO")))
    headers.append((b"X-Request-Data", request.body))

    return headers, b"Test"
