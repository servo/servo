import os

from wptserve.utils import isomorphic_decode

def main(request, response):
    stash_id = request.GET.first(b"id")
    if request.server.stash.take(stash_id) is not None:
        response.status = (404, u"Not Found")
        response.headers.set(b"Content-Type", b"text/plain")
        return u"not found"
    request.server.stash.put(stash_id, True)

    path = os.path.join(os.path.dirname(isomorphic_decode(__file__)), u"sxg", u"sxg-prefetch-test.sxg")
    body = open(path, u"rb").read()

    response.headers.set(b"Content-Type", b"application/signed-exchange;v=b3")
    response.headers.set(b"X-Content-Type-Options", b"nosniff")
    response.headers.set(b"Cache-Control", b"public, max-age=600")

    return body.replace(b'XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX', stash_id)
