import time


def main(request, response):
    action = request.GET.first(b"action", None)
    if action == b"continue":
        key = request.GET.first(b"key")
        request.server.stash.take(key)
        request.server.stash.put(key, "continue")
        return (200, [(b"Content-Type", b"text/plain")], b"ok")

    if action == b"check_chunk1":
        key = request.GET.first(b"key")
        val = request.server.stash.take(key)
        if val == "chunk1_sent":
            request.server.stash.put(key, "chunk1_sent")
            return (200, [(b"Content-Type", b"text/plain")], b"ready")
        elif val is not None:
            request.server.stash.put(key, val)
        return (200, [(b"Content-Type", b"text/plain")], b"waiting")

    content1 = request.GET.first(b"chunk1", b"")
    content2 = request.GET.first(b"chunk2", b"")
    key = request.GET.first(b"key", None)

    response.headers.set(b"Content-Type", b"text/html")
    if request.GET.first(b"cors", b"0") == b"1":
        response.headers.set(b"Access-Control-Allow-Origin", b"*")
    response.write_status_headers()
    response.writer.write_content(content1)

    if key:
        request.server.stash.put(key, "chunk1_sent")

        for _ in range(200):
            val = request.server.stash.take(key)
            if val == "continue":
                break
            elif val is not None:
                request.server.stash.put(key, val)
            time.sleep(0.05)
    else:
        delay = float(request.GET.first(b"delay", 300)) / 1000.0
        time.sleep(delay)

    response.writer.write_content(content2)
