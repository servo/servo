from wptserve.utils import isomorphic_decode

def main(request, response):
    response.headers.set(b"Cache-Control", b"no-store")

    # This should be a simple request; deny preflight
    if request.method != u"POST":
        response.status = 400
        return

    response.headers.set(b"Access-Control-Allow-Credentials", b"true")
    response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))

    for header in [b"Accept", b"Accept-Language", b"Content-Language", b"Content-Type"]:
        value = request.headers.get(header)
        response.content += isomorphic_decode(header) + u": " + (isomorphic_decode(value) if value else u"<None>") + u'\n'
