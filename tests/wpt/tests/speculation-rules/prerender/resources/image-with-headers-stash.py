# Supports two operations:
# - ?image=uuid: Returns an image, and records the request headers that were
#   used to get that image.
# - ?read=uuid: Returns the request headers in the stash keyed by a given uuid.

import os
import json

from wptserve.utils import isomorphic_decode

def main(request, response):
  if b"image" in request.GET:
    uuid = request.GET[b"image"]

    header_pairs = []
    for header_name in request.headers.keys():
        # ensure the header name/value are unicode strings
        name = header_name.lower().decode("utf-8")
        for header_value in request.headers.get_list(header_name):
            value = header_value.decode("utf-8")
            header_pairs.append([name, value])

    request_headers = json.dumps(header_pairs)
    request.server.stash.put(uuid, request_headers)

    # Return a basic image.
    response_headers = [
      (b"Content-Type", b"image/png"),
      (b"Access-Control-Allow-Origin", b"*")
    ]
    image_path = os.path.join(
      os.path.dirname(isomorphic_decode(__file__)),
      u"../../../common/square.png"
    )
    return (200, response_headers, open(image_path, mode='rb').read())

  elif b"read" in request.GET:
    uuid = request.GET[b"read"]
    stash_value = request.server.stash.take(uuid)

    if stash_value is None:
      stash_value = "null"
    return (200, [(b"Content-Type", b"application/json")], str(stash_value))

  return (404 , [], "Not found")
