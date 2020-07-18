from wptserve.utils import isomorphic_decode

def main(request, response):
    token = request.GET.first(b"token", None)
    if b"querystate" in request.GET:
        from json import JSONEncoder
        response.headers.set(b"Content-Type", b"text/plain")
        return JSONEncoder().encode(request.server.stash.take(token))
    content = request.GET.first(b"content", None)
    tag = request.GET.first(b"tag", None)
    date = request.GET.first(b"date", None)
    expires = request.GET.first(b"expires", None)
    vary = request.GET.first(b"vary", None)
    cc = request.GET.first(b"cache_control", None)
    redirect = request.GET.first(b"redirect", None)
    inm = request.headers.get(b"If-None-Match", None)
    ims = request.headers.get(b"If-Modified-Since", None)
    pragma = request.headers.get(b"Pragma", None)
    cache_control = request.headers.get(b"Cache-Control", None)
    ignore = b"ignore" in request.GET

    if tag:
        tag = b'"%s"' % tag

    server_state = request.server.stash.take(token)
    if not server_state:
        server_state = []
    state = dict()
    if not ignore:
        if inm:
            state[u"If-None-Match"] = isomorphic_decode(inm)
        if ims:
            state[u"If-Modified-Since"] = isomorphic_decode(ims)
        if pragma:
            state[u"Pragma"] = isomorphic_decode(pragma)
        if cache_control:
            state[u"Cache-Control"] = isomorphic_decode(cache_control)
    server_state.append(state)
    request.server.stash.put(token, server_state)

    if tag:
        response.headers.set(b"ETag", b'%s' % tag)
    elif date:
        response.headers.set(b"Last-Modified", date)
    if expires:
        response.headers.set(b"Expires", expires)
    if vary:
        response.headers.set(b"Vary", vary)
    if cc:
        response.headers.set(b"Cache-Control", cc)

    # The only-if-cached redirect tests wants CORS to be okay, the other tests
    # are all same-origin anyways and don't care.
    response.headers.set(b"Access-Control-Allow-Origin", b"*")

    if redirect:
        response.headers.set(b"Location", redirect)
        response.status = (302, b"Redirect")
        return b""
    elif ((inm is not None and inm == tag) or
          (ims is not None and ims == date)):
        response.status = (304, b"Not Modified")
        return b""
    else:
        response.status = (200, b"OK")
        response.headers.set(b"Content-Type", b"text/plain")
        return content
