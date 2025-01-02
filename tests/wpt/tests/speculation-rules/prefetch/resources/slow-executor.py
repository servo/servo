import os.path
import time

from wptserve.pipes import template

def main(request, response):
    time.sleep(float(request.GET.first(b"delay")))
    response.headers.set(b"Content-Type", b"text/html")
    response.headers.set(b"Cache-Control", b"no-store")
    response.content = template(
        request,
        open(os.path.join(os.path.dirname(__file__), "executor.sub.html"), "rb").read())

