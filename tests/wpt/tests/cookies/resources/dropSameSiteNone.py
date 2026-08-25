from cookies.resources.helpers import makeDropCookie, setNoCacheAndCORSHeaders

def main(request, response):
    """Respond to `/cookies/resources/dropSameSiteNone.py by dropping the
    two cookies set by setSameSiteNone.py"""
    headers = setNoCacheAndCORSHeaders(request, response)

    # Expire the cookies, and return a JSON-encoded success code.
    headers.append(makeDropCookie(b"samesite_none_insecure", False))
    headers.append(makeDropCookie(b"samesite_none_secure", True))
    return headers, b'{"success": true}'
