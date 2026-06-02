import json

def main(request, response):
  response.headers.set(b"Content-Type", b"application/json")
  response.headers.set(b"Cache-Control", b"no-store")

  uuid = request.GET[b"uuid"]

  with request.server.stash.lock:
    request_count = request.server.stash.take(uuid)
    if request_count is None:
      request_count = 0

    if not b"check" in request.GET:
      request_count += 1

    request.server.stash.put(uuid, request_count)

  return json.dumps(request_count)
