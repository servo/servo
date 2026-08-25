def main(request, response):
    if request.method == u'POST':
        request.server.stash.put(request.GET[b"id"], request.body)
        return u''
    return request.server.stash.take(request.GET[b"id"])
