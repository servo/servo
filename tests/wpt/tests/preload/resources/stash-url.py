def main(request, response):
    response.headers.set(b'Content-Type', b'application/json')
    response.headers.set(b'Cache-Control', b'no-store')

    key = request.GET.first(b'key')
    request.server.stash.put(key, request.url, path="preload/resources/")
    return b'{"ok": true}'
