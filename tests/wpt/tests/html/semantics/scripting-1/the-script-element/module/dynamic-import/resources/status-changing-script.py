def main(request, response):
    headers = [(b"Content-Type", b"text/javascript"),
               (b"Cache-Control", b"private, no-store")]

    id = request.GET.first(b"id")

    with request.server.stash.lock:
        status = request.server.stash.take(id)
        if status is None:
            status = 200

        new_status = request.GET.first(b"newStatus", None)
        if new_status is not None:
            status = int(new_status)

        request.server.stash.put(id, status)

        return status, headers, b""
