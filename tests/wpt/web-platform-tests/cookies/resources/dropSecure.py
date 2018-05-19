from helpers import makeDropCookie, readParameter, setNoCacheAndCORSHeaders

def main(request, response):
    """Respond to `/cookie/drop/secure` by dropping the two cookie set by
    `setSecureTestCookies()`"""
    headers = setNoCacheAndCORSHeaders(request, response)

    # Expire the cookies, and return a JSON-encoded success code.
    headers.append(makeDropCookie("alone_secure", False))
    headers.append(makeDropCookie("alone_insecure", False))
    return headers, '{"success": true}'
