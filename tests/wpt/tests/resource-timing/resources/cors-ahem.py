import os.path

from wptserve.utils import isomorphic_decode

def main(request, response):
    etag = b"123abc"
    if etag == request.headers.get(b"If-None-Match", None):
        response.headers.set(b"X-HTTP-STATUS", 304)
        response.status = (304, b"Not Modified")
        return u""

    response.headers.set(b"Cache-Control", b"public, max-age=86400")
    response.headers.set(b"Content-Type", b"font/truetype")
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    response.headers.set(b"Timing-Allow-Origin", b"*")
    response.headers.set(b"ETag", etag)
    font = u"../../fonts/Ahem.ttf"
    path = os.path.join(os.path.dirname(isomorphic_decode(__file__)), font)
    response.content = open(path, u"rb").read()
