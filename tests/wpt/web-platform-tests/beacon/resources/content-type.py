def main(request, response):
    command = request.GET.first(b"cmd").lower()
    test_id = request.GET.first(b"id")
    if command == b"put":
        request.server.stash.put(test_id, request.headers.get(b"Content-Type", b""))
        return [(b"Content-Type", b"text/plain")], u""

    if command == b"get":
        stashed_header = request.server.stash.take(test_id)
        if stashed_header is not None:
            return [(b"Content-Type", b"text/plain")], stashed_header

    response.set_error(400, u"Bad Command")
    return u"ERROR: Bad Command!"
