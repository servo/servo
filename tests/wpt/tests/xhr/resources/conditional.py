def main(request, response):
    tag = request.GET.first(b"tag", None)
    match = request.headers.get(b"If-None-Match", None)
    date = request.GET.first(b"date", b"")
    modified = request.headers.get(b"If-Modified-Since", None)
    cors = request.GET.first(b"cors", None)

    if request.method == u"OPTIONS":
        response.headers.set(b"Access-Control-Allow-Origin", b"*")
        response.headers.set(b"Access-Control-Allow-Headers", b"IF-NONE-MATCH")
        return b""

    if tag:
        response.headers.set(b"ETag", b'"%s"' % tag)
    elif date:
        response.headers.set(b"Last-Modified", date)

    if cors:
        response.headers.set(b"Access-Control-Allow-Origin", b"*")

    if ((match is not None and match == tag) or
            (modified is not None and modified == date)):
        response.status = (304, b"SUPERCOOL")
        return b""
    else:
        if not cors:
            response.headers.set(b"Access-Control-Allow-Origin", b"*")
        response.headers.set(b"Content-Type", b"text/plain")
        return b"MAYBE NOT"
