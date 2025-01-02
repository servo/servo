def main(request, response):
    headers = [(b"Content-Type", b"text/plain")]
    headers.append((b"Access-Control-Allow-Origin", b"*"))

    if request.method == u"GET":
        token = request.GET.first(b"token")
        value = request.server.stash.take(token)
        if value == None:
            body = u"0"
        else:
            if request.GET.first(b"check", None) == b"keep":
                request.server.stash.put(token, value)
            body = u"1"

        return headers, body

    if request.method == u"OPTIONS":
        if not b"Access-Control-Request-Method" in request.headers:
            response.set_error(400, u"No Access-Control-Request-Method header")
            return u"ERROR: No access-control-request-method in preflight!"

        headers.append((b"Access-Control-Allow-Methods",
                        request.headers[b'Access-Control-Request-Method']))

        if b"max_age" in request.GET:
            headers.append((b"Access-Control-Max-Age", request.GET[b'max_age']))

        if b"token" in request.GET:
            request.server.stash.put(request.GET.first(b"token"), 1)

    headers.append((b"Access-Control-Allow-Headers", b"x-print"))

    body = request.headers.get(b"x-print", b"NO")

    return headers, body
