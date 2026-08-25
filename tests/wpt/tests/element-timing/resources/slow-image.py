import os.path
import time

from wptserve.utils import isomorphic_encode

def main(request, response):
    name = request.GET.first(b"name")
    sleepTime = float(request.GET.first(b"sleep")) / 1E3

    time.sleep(sleepTime)

    path = os.path.join(os.path.dirname(isomorphic_encode(__file__)), name)
    body = open(path, u"rb").read()

    response.headers.set(b"Content-Type", b"image")
    response.headers.set(b"Content-Length", len(body))
    response.headers.set(b"Cache-control", b"no-cache, must-revalidate")

    response.content = body;
