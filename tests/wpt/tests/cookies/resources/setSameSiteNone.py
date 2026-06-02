from cookies.resources.helpers import makeCookieHeader, setNoCacheAndCORSHeaders

from wptserve.utils import isomorphic_encode

def main(request, response):
    """Respond to `/cookies/resources/setSameSiteNone.py?{value}` by setting two cookies:
    1. `samesite_none_insecure={value};SameSite=None;path=/`
    2. `samesite_none_secure={value};SameSite=None;Secure;path=/`
    """
    headers = setNoCacheAndCORSHeaders(request, response)
    value = isomorphic_encode(request.url_parts.query)

    headers.append(makeCookieHeader(b"samesite_none_insecure", value, {b"SameSite":b"None", b"path":b"/"}))
    headers.append(makeCookieHeader(b"samesite_none_secure", value, {b"SameSite":b"None", b"Secure":b"", b"path":b"/"}))

    return headers, b'{"success": true}'
