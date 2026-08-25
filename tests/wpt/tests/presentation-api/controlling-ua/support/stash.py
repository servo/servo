def main(request, response):
    key = request.GET.first(b"id")

    if request.method == u"POST":
        request.server.stash.put(key, request.body)
        return b"ok"
    else:
        value = request.server.stash.take(key)
        assert request.server.stash.take(key) is None
        return value