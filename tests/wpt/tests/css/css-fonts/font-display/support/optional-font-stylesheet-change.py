import os
import time


def main(request, response):
    key = request.GET.first(b"key")
    if request.method == "POST":
        request.server.stash.put(key, True)
        return b""

    while request.server.stash.take(key) is None:
        time.sleep(0.01)

    response.headers.set(b"Content-Type", b"font/ttf")
    response.headers.set(b"Cache-Control", b"max-age=3600")
    with open(os.path.join(request.doc_root, "fonts", "Ahem.ttf"), "rb") as font:
        return font.read()
