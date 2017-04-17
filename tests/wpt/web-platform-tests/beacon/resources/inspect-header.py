def main(request, response):
    headers = [("Content-Type", "text/plain")]
    command = request.GET.first("cmd").lower();
    test_id = request.GET.first("id")
    header = request.GET.first("header")
    if command == "put":
        request.server.stash.put(test_id, request.headers.get(header, ""))

    elif command == "get":
        stashed_header = request.server.stash.take(test_id)
        if stashed_header is not None:
            headers.append(("x-request-" + header, stashed_header ))

    else:
        response.set_error(400, "Bad Command")
        return "ERROR: Bad Command!"

    return headers, ""
