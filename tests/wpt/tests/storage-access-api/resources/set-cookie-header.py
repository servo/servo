from urllib.parse import unquote
from wptserve.utils import isomorphic_encode

def main(request, response):
  # Cookies may require whitespace (e.g. in the `Expires` attribute), so the
  # query string should be decoded.
  cookie = unquote(request.url_parts.query)
  headers = []
  headers.append((b"Set-Cookie", isomorphic_encode(cookie)))

  return (200, headers, "")

