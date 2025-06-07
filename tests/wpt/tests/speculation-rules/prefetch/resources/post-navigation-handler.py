import os
from wptserve.pipes import template

# This is used only to accept POST navigations.
def main(request, response):
    response.headers.set(b"Content-Type", b"text/html")
    response.headers.set(b"Cache-Control", b"no-store")
    response.content = template(
    request,
    open(os.path.join(os.path.dirname(__file__), "executor.sub.html"), "rb").read())