import time


def main(request, response):
    action = request.GET.first(b"action", None)
    key = request.GET.first(b"key")

    if action == b"continue":
        request.server.stash.take(key)
        request.server.stash.put(key, "continue")
        return (
            200,
            [(b"Content-Type", b"text/plain"), (b"Cache-Control", b"no-store")],
            b"ok",
        )

    content_type = request.GET.first(b"content_type", b"text/html; charset=utf-8")
    chunk1 = request.GET.first(b"chunk1", b"")
    chunk2 = request.GET.first(b"chunk2", b"")

    response.headers.set(b"Content-Type", content_type)
    response.headers.set(b"Cache-Control", b"no-store")
    response.headers.set(b"X-Content-Type-Options", b"nosniff")
    response.write_status_headers()
    response.writer.write_content(chunk1)

    for _ in range(250):
        if request.server.stash.take(key) == "continue":
            break
        time.sleep(0.02)

    response.writer.write_content(chunk2)
