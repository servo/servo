from wptserve.handlers import json_handler

@json_handler
def main(request, response):
  uuid = request.GET[b"uuid"]
  prefetch = request.headers.get(
      "Sec-Purpose", b"").decode("utf-8").startswith("prefetch")
  response.headers.set("Cache-Control", "no-store")

  n = request.server.stash.take(uuid)
  if n is None:
    n = 0
  if prefetch:
    n += 1
    request.server.stash.put(uuid, n)

  return n
