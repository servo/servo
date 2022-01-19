from cookies.resources import helpers
from urllib.parse import unquote

from wptserve.utils import isomorphic_encode

def main(request, response):
    """Respond to `/cookie/set?{cookie}` by echoing `{cookie}` as a `Set-Cookie` header."""
    headers = helpers.setNoCacheAndCORSHeaders(request, response)

    # Cookies may require whitespace (e.g. in the `Expires` attribute), so the
    # query string should be decoded.
    cookie = unquote(request.url_parts.query)
    headers.append((b"Set-Cookie", isomorphic_encode(cookie)))

    return headers, b'{"success": true}'
