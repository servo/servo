from cookies.resources.helpers import makeDropCookie, setNoCacheAndCORSHeaders

def main(request, response):
    """Respond to `/cookies/resources/dropSameSiteMultiAttribute.py by dropping
    the cookies set by setSameSiteMultiAttribute.py"""
    headers = setNoCacheAndCORSHeaders(request, response)

    # Expire the cookies, and return a JSON-encoded success code.
    headers.append(makeDropCookie(b"samesite_unsupported", True))
    headers.append(makeDropCookie(b"samesite_unsupported_none", True))
    headers.append(makeDropCookie(b"samesite_unsupported_lax", False))
    headers.append(makeDropCookie(b"samesite_unsupported_strict", False))
    headers.append(makeDropCookie(b"samesite_none_unsupported", True))
    headers.append(makeDropCookie(b"samesite_lax_unsupported", True))
    headers.append(makeDropCookie(b"samesite_strict_unsupported", True))
    headers.append(makeDropCookie(b"samesite_lax_none", True))
    return headers, b'{"success": true}'
