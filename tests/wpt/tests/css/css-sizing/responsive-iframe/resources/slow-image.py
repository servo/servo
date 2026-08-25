import os.path
import time

def main(request, response):
    # Sleep for 500ms to delay the `load` events.
    time.sleep(0.5)

    name = request.GET.first(b"name")
    path = os.path.join(os.path.dirname(__file__), name)
    body = open(path, u"rb").read()

    response.headers.set(b"Content-Type", b"image")
    response.headers.set(b"Content-Length", len(body))
    response.headers.set(b"Cache-control", b"no-cache, must-revalidate")

    response.content = body
