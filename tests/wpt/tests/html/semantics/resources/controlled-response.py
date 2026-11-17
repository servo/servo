import time


def main(request, response):
    key = request.GET.first(b"key")
    path = request.url_parts.path
    action = request.GET.first(b"action", b"response")
    if action == b"release":
        request.server.stash.put(key, True, path)
        return b"released"

    response.headers.set(b"Content-Type", request.GET.first(b"type"))
    response.headers.set(b"Cache-Control", b"no-store")
    # The test controls completion; this deadline only bounds abandoned server handlers.
    deadline = time.monotonic() + 30
    while request.server.stash.take(key, path) is None:
        if time.monotonic() >= deadline:
            response.status = 504
            return b"The test did not release this response"
        time.sleep(0.01)
    return request.GET.first(b"body")
