def main(request, response):
    headers = [(b"Content-Type", b"text/plain")]
    command = request.GET.first(b"cmd").lower()
    test_id = request.GET.first(b"id")
    header = request.GET.first(b"header")
    if command == b"put":
        request.server.stash.put(test_id, request.headers.get(header, b""))

    elif command == b"get":
        stashed_header = request.server.stash.take(test_id)
        if stashed_header is not None:
            headers.append((b"x-request-" + header, stashed_header))

    else:
        response.set_error(400, u"Bad Command")
        return b"ERROR: Bad Command!"

    return headers, b""
