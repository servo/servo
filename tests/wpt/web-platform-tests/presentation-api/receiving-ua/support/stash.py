def main(request, response):
    key = request.GET.first("id")

    if request.method == "POST":
        request.server.stash.put(key, request.body)
        return "ok"
    else:
        value = request.server.stash.take(key)
        assert request.server.stash.take(key) is None
        return value