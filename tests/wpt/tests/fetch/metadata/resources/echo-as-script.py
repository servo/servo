import json

from wptserve.utils import isomorphic_decode

def main(request, response):
    headers = [(b"Content-Type", b"text/javascript")]
    body = u"var header = %s;" % json.dumps({
        u"dest": isomorphic_decode(request.headers.get(b"sec-fetch-dest", b"")),
        u"mode": isomorphic_decode(request.headers.get(b"sec-fetch-mode", b"")),
        u"site": isomorphic_decode(request.headers.get(b"sec-fetch-site", b"")),
        u"user": isomorphic_decode(request.headers.get(b"sec-fetch-user", b"")),
        })

    return headers, body
