def main(request, response):
    response.headers.set(b'Content-Type', b'text/plain')

    key = request.GET.first(b'key')
    request.server.stash.put(key, request.url, path="/preload/resources/")
    return b'ok'