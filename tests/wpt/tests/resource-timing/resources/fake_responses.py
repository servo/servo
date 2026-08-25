# /xhr/resources/conditional.py -- to fake a 304 response

def main(request, response):
    if request.method == "OPTIONS":
        # Assume this is a CORS preflight
        response.headers.set(b"Access-Control-Allow-Headers", "*")
        response.headers.set(b"Access-Control-Allow-Origin", "*")
        response.status = (204, "No Content")
        return b""
    tag = request.GET.first(b"tag", None)
    redirect = request.GET.first(b"redirect", None)
    match = request.headers.get(b"If-None-Match", None)
    date = request.GET.first(b"date", b"")
    modified = request.headers.get(b"If-Modified-Since", None)
    url = request.GET.first(b"url", None)
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    response.headers.set(b"Timing-Allow-Origin", b"*")
    if tag:
        response.headers.set(b"ETag", b'"%s"' % tag)
    elif date:
        response.headers.set(b"Last-Modified", date)
    if redirect:
        response.headers.set(b"Location", redirect)
        response.status = (302, b"Moved")
        return b""

    if url:
        filename = url.decode('utf-8').split("?")[0]
        filepath = "./resource-timing/resources/{}".format(filename)
        response.headers.set(b"Content-Type", b"text/javascript")
        with open(filepath, 'rb') as f:
            filedata = f.read()

        return filedata

    if ((match is not None and match == tag) or
        (modified is not None and modified == date)):
        response.status = (304, b"SUPERCOOL")
        return b""
    else:
        response.headers.set(b"Content-Type", b"text/plain")
        return b"MAYBE NOT"
