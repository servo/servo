import os

from wptserve.utils import isomorphic_decode

# Returns a valid image response when request's |referrer| matches
# |expected_referrer|.
def main(request, response):
  referrer = request.headers.get(b"referer", b"")
  expected_referrer = request.GET.first(b"expected_referrer", b"")
  response_headers = [(b"Content-Type", b"image/png")]
  if referrer == expected_referrer:
    image_path = os.path.join(os.path.dirname(isomorphic_decode(__file__)), u"image.png")
    return (200, response_headers, open(image_path, mode='rb').read())
  return (404, response_headers, u"Not found")
