def main(request, response):
    """Helper handler for testing the URL encoding used by sendBeacon().

    The 'store' command stashes the request's raw (percent-encoded) query
    string keyed by the 'id' UUID query parameter. The 'stat' command takes
    that stashed string back out.
    """
    cmd = request.GET.first(b"cmd")
    id = request.GET.first(b"id")

    if cmd == b"store":
        request.server.stash.put(id, request.url_parts.query)
    elif cmd == b"stat":
        stored = request.server.stash.take(id)
        response.headers.set(b"Content-Type", b"text/plain")
        response.content = stored.encode("ascii") if stored is not None else b""
    else:
        response.status = 400
