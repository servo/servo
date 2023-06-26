def main(request, response):
    key = request.GET[b'key']
    already_requested = request.server.stash.take(key)

    if already_requested is None:
        request.server.stash.put(key, True)
        return [(b'Content-Type', b'application/javascript')], b'// initial script'

    response.status = (404, b'Not found: should not have been able to import this script twice!')
