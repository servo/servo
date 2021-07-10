
def main(request, response):
    origin = request.GET.first(b"origin", request.headers[b"origin"])
    credentials = request.GET.first(b"credentials", b"true")

    headers = [(b"Content-Type", b"text/plain")]
    if origin != b'none':
        headers.append((b"Access-Control-Allow-Origin", origin))
    if credentials != b'none':
        headers.append((b"Access-Control-Allow-Credentials", credentials))

    ident = request.GET.first(b'ident', b'test')

    if ident in request.cookies:
        body = request.cookies[ident].value
        response.delete_cookie(ident)
    else:
        response.set_cookie(ident, b"COOKIE")
        body = u"NO_COOKIE"

    return headers, body
