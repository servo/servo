# This is a simple implementation of a server-side stash that supports two
# operations:
#   - increment:
#       Increments a value in the stash keyed by a given uuid, and returns an
#       image file
#   - read:
#       Returns the value in the stash keyed by a given uuid or 0 otherwise.
#       This is a read-only operation, and does not remove the value from the
#       stash as-is the default WPT stash behavior:
#       https://web-platform-tests.org/tools/wptserve/docs/stash.html.

import os

from wptserve.utils import isomorphic_decode

def main(request, response):
  if b"increment" in request.GET:
    uuid = request.GET[b"increment"]

    # First, increment the stash value keyed by `uuid`, and write it back to the
    # stash. Writing it back to the stash is necessary since `take()` actually
    # removes the value whereas we want to increment it.
    stash_value = request.server.stash.take(uuid)
    if stash_value is None:
      stash_value = 0
    request.server.stash.put(uuid, stash_value + 1)

    # Return a basic image.
    response_headers = [(b"Content-Type", b"image/png")]
    image_path = os.path.join(os.path.dirname(isomorphic_decode(__file__)), u"image.png")
    return (200, response_headers, open(image_path, mode='rb').read())

  elif b"read" in request.GET:
    uuid = request.GET[b"read"]
    stash_value = request.server.stash.take(uuid)

    if stash_value is None:
      stash_value = 0
    # Write the stash value keyed by `uuid` back to the stash. This is necessary
    # because `take()` actually removes it, but we want a read-only read.
    request.server.stash.put(uuid, stash_value);
    return (200, [], str(stash_value))

  return (404 , [], "Not found")
