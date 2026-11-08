import json

from wptserve.utils import isomorphic_decode


def main(request, response):
    key = request.GET[b"id"]
    if request.method == "POST":
        request.server.stash.put(key, {
            "method": request.method,
            "contentType": isomorphic_decode(request.headers.get(b"content-type", b"")),
            "body": isomorphic_decode(request.body),
            "pingTo": isomorphic_decode(request.headers.get(b"ping-to", b"")),
        })
        status = int(request.GET[b"status"])
        return status, [
            (b"Content-Type", request.GET[b"type"]),
            (b"X-Content-Type-Options", b"nosniff"),
            (b"Cache-Control", b"no-store"),
        ], b'{"ping": true}' if status != 204 else b""

    return 200, [
        (b"Content-Type", b"application/json"),
        (b"Cache-Control", b"no-store"),
    ], json.dumps(request.server.stash.take(key))
