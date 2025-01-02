import json
from wptserve.utils import isomorphic_decode

def main(request, response):
    key = request.GET[b"id"]

    if request.method == "POST":
      content_type = request.headers.get(b"content-type", b"no content-type header")
      ping_from = request.headers.get(b"ping-from", b"no ping-from header")
      ping_to = request.headers.get(b"ping-to", b"no ping-to header")

      value = json.dumps({
        'content-type': isomorphic_decode(content_type),
        'ping-from': isomorphic_decode(ping_from),
        'ping-to': isomorphic_decode(ping_to)
      })
      request.server.stash.put(key, value)

      return (204, [], "")

    elif request.method == "GET":
      value = request.server.stash.take(key)
      if value is None:
        value = "\"no headers yet\""
      return (200, [("Content-Type", "application/json")], str(value))

    return (405, [], "")
