import json
import os.path
import time
from wptserve.pipes import template

def main(request, response):
  response.headers.set(b"Content-Type", b"text/html")
  response.headers.set(b"Cache-Control", b"no-store")

  uuid = request.GET[b"uuid"]

  # The lock is needed because the server receives concurrent requests to the
  # same `uuid` e.g. due to `race-network-and-fetch-handler`.
  with request.server.stash.lock:
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

  # This delay is after the `stash.put` above so that the request is counted
  # upon received (i.e. we don't have to wait for the delay completion).
  if b"delay" in request.GET:
    time.sleep(float(request.GET[b"delay"]) / 1E3)

  if b"location" in request.GET:
    response.status = 302
    response.headers.set(b"Location", request.GET[b"location"])
    return

  response.content = template(
    request,
    open(os.path.join(os.path.dirname(__file__), "executor.sub.html"), "rb").read())
