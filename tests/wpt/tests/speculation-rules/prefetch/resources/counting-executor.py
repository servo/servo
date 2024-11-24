import json
import os.path
from wptserve.pipes import template

def main(request, response):
  response.headers.set(b"Content-Type", b"text/html")
  response.headers.set(b"Cache-Control", b"no-store")

  uuid = request.GET[b"uuid"]
  request_count = request.server.stash.take(uuid)
  if request_count is None:
    request_count = {"prefetch": 0, "nonPrefetch": 0}

  if b"check" in request.GET:
    response.content = json.dumps(request_count)
    return

  prefetch = request.headers.get(
    "Sec-Purpose", b"").decode("utf-8").startswith("prefetch")

  request_count["prefetch" if prefetch else "nonPrefetch"] += 1
  request.server.stash.put(uuid, request_count)

  response.content = template(
    request,
    open(os.path.join(os.path.dirname(__file__), "executor.sub.html"), "rb").read())
