def main(request, response):
    key = request.GET['key']
    already_requested = request.server.stash.take(key)

    if already_requested is None:
        request.server.stash.put(key, True)
        return [('Content-Type', 'application/javascript')], '// initial script'

    response.status = (404, 'Not found: should not have been able to import this script twice!')
