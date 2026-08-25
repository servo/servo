import json

from wptserve.utils import isomorphic_decode

def main(request, response):
    headers = [(b"Content-Type", b"application/json"),
               (b"Access-Control-Allow-Credentials", b"true")]

    if b"origin" in request.headers:
        headers.append((b"Access-Control-Allow-Origin", request.headers[b"origin"]))

    body = u""

    # If we're in a preflight, verify that `Sec-Fetch-Mode` is `cors`.
    if request.method == u'OPTIONS':
        if request.headers.get(b"sec-fetch-mode") != b"cors":
            return (403, b"Failed"), [], body

        headers.append((b"Access-Control-Allow-Methods", b"*"))
        headers.append((b"Access-Control-Allow-Headers", b"*"))
    else:
        body = json.dumps({
            u"dest": isomorphic_decode(request.headers.get(b"sec-fetch-dest", b"")),
            u"mode": isomorphic_decode(request.headers.get(b"sec-fetch-mode", b"")),
            u"site": isomorphic_decode(request.headers.get(b"sec-fetch-site", b"")),
            u"user": isomorphic_decode(request.headers.get(b"sec-fetch-user", b"")),
            })

    return headers, body
