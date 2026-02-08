import os

from wptserve.utils import isomorphic_decode

STASH_ID = "882e0112-2a43-4ff1-9503-658b1a2a8e67"

def main(request, response):
  # Alternate between blue-10.png and green.png on each request
  switch = request.server.stash.take(STASH_ID)
  request.server.stash.put(STASH_ID, not switch)

  image = "blue-10.png" if switch else "green.png"
  response_headers = [
      (b"Content-Type", b"image/png"),
      (b"Cache-Control", b"no-store")
  ]
  image_path = os.path.join(os.path.dirname(isomorphic_decode(__file__)), image)
  return (200, response_headers, open(image_path, mode='rb').read())
