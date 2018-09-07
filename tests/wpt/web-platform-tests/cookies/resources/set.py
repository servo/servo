import helpers
import urllib

def main(request, response):
    """Respond to `/cookie/set?{cookie}` by echoing `{cookie}` as a `Set-Cookie` header."""
    headers = helpers.setNoCacheAndCORSHeaders(request, response)

    # Cookies may require whitespace (e.g. in the `Expires` attribute), so the
    # query string should be decoded.
    cookie = urllib.unquote(request.url_parts.query)
    headers.append(("Set-Cookie", cookie))

    return headers, '{"success": true}'
