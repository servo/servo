import os.path
import time

from wptserve.utils import isomorphic_encode

def main(request, response):
    name = request.GET.first(b"name")
    sleepTime = float(request.GET.first(b"sleep")) / 1E3
    numInitial = int(request.GET.first(b"numInitial"))

    path = os.path.join(os.path.dirname(isomorphic_encode(__file__)), name)
    body = open(path, u"rb").read()

    response.headers.set(b"Content-Type", b"image")
    response.headers.set(b"Content-Length", len(body))
    response.headers.set(b"Cache-control", b"no-cache, must-revalidate")
    response.write_status_headers()

    # Read from the beginning, |numInitial| bytes.
    first = body[:numInitial]
    response.writer.write_content(first)

    time.sleep(sleepTime)

    # Read the remainder after having slept.
    second = body[numInitial:]
    response.writer.write_content(second)
