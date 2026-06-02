from wptserve.utils import isomorphic_encode

def main(request, response):
    response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))
    response.headers.set(b"Access-Control-Expose-Headers", b"X-Request-Method")

    if request.method == u'OPTIONS':
        response.headers.set(b"Access-Control-Allow-Methods",  b"GET, CHICKEN, HEAD, POST, PUT")

    if b'headers' in request.GET:
        response.headers.set(b"Access-Control-Allow-Headers",  request.GET.first(b'headers'))

    response.headers.set(b"X-Request-Method", isomorphic_encode(request.method))

    response.headers.set(b"X-A-C-Request-Method", request.headers.get(b"Access-Control-Request-Method", b""))


    #This should reasonably work for most response codes.
    try:
        code = int(request.GET.first(b"code", 200))
    except ValueError:
        code = 200

    text = request.GET.first(b"text", b"OMG")

    if request.method == u"OPTIONS" and b"preflight" in request.GET:
        try:
            code = int(request.GET.first(b'preflight'))
        except KeyError:
            pass

    status = code, text

    if b"type" in request.GET:
        response.headers.set(b"Content-Type", request.GET.first(b'type'))

    body = request.GET.first(b'content', b"")

    return status, [], body
