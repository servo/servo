from wptserve.handlers import HTTPException

def main(request, response):
    if request.method != u"GET":
        raise HTTPException(400, message=u"Method was not GET")

    if not b"id" in request.GET:
        raise HTTPException(400, message=u"No id")

    id = request.GET[b'id']
    if b"read" in request.GET:
        data = request.server.stash.take(id)
        if data is None:
            response.set_error(404, u"Tried to read data not yet set")
            return
        return [(b"Content-Type", b"text/plain")], data

    elif b"cleanup" in request.GET:
        request.server.stash.take(id)
        return b"OK"

    elif b"delete-cookie" in request.GET:
        response.delete_cookie(id)
        return [(b"Content-Type", b"text/plain")], b"OK"

    if b"origin" in request.GET:
        response.headers.set(b'Access-Control-Allow-Origin', request.GET[b'origin'])
        response.headers.set(b'Access-Control-Allow-Credentials', b'true')

    cors = request.headers.get(b"origin", b"no")

    cookie = request.cookies.first(id, None)
    cookie_value = cookie.value if cookie is not None else b"no"

    line = b'cors = ' + cors + b' | cookie = ' + cookie_value

    data = request.server.stash.take(id)
    if data is not None:
        line = data + b"\n" + line

    request.server.stash.put(id, line)

    if b"redirect" in request.GET:
        response.status = 302
        response.headers.set(b'Location', request.GET[b'redirect'])
    else:
        return b"""WEBVTT

00:00:00.000 --> 00:00:10.000
Test"""
