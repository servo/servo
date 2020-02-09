import os

# Returns a valid image response when request's |referrer| matches
# |expected_referrer|.
def main(request, response):
  referrer = request.headers.get("referer", "")
  expected_referrer = request.GET.first("expected_referrer", "")
  response_headers = [("Content-Type", "image/png")]
  if referrer == expected_referrer:
    image_path = os.path.join(os.path.dirname(__file__), "image.png")
    return (200, response_headers, open(image_path, mode='rb').read())
  return (404, response_headers, "Not found")
