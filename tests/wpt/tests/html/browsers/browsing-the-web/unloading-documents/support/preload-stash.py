import json
import time

def main(request, response):
  uuid = request.GET[b"uuid"]
  response.headers.set(b"Cache-Control", b"no-store")

  with request.server.stash.lock:
    received = request.server.stash.take(uuid)

    if b"check" in request.GET:
      response.headers.set(b"Content-Type", b"application/json")
      response.content = json.dumps({"received": received is not None and received > 0})
      if received:
        request.server.stash.put(uuid, received)
      return

    if received is None:
      received = 0
    received += 1
    request.server.stash.put(uuid, received)

  response.headers.set(b"Content-Type", b"application/javascript")
  response.content = b"// preloaded resource"
