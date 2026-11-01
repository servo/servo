import json
import os.path
import time

from wptserve.pipes import template

def main(request, response):
  response.headers.set(b"Content-Type", b"text/html")
  response.headers.set(b"Cache-Control", b"no-store")

  prefetch = request.headers.get("Sec-Purpose", b"").decode("utf-8").startswith("prefetch")

  response.content = template(
    request,
    open(os.path.join(os.path.dirname(__file__), "executor.sub.html"), "rb").read())

  if prefetch:
    # Delay the prefetch response so the navigation can start while the
    # prefetch is still in-flight, causing them to share the same resource.
    time.sleep(1)
    response.status = 503
    response.content += b"<body>503"
  else:
    response.status = 200
    response.content += b"<body>200"
