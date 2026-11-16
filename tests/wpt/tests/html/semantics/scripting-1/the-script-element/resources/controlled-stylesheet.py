import time


def main(request, response):
    token = request.GET[b"token"]
    response.headers.set(b"Cache-Control", b"no-store")

    if request.method == "POST":
        request.server.stash.put(token, True)
        return b""

    response.headers.set(b"Content-Type", b"text/css")
    response.write_status_headers()

    # Keep the stylesheet pending until released, or the client disconnects.
    while request.server.stash.take(token) is None:
        if not response.writer.write(b"\n"):
            return
        time.sleep(0.01)
