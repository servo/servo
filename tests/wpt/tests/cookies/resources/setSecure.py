from cookies.resources.helpers import makeCookieHeader, readParameter, setNoCacheAndCORSHeaders

from wptserve.utils import isomorphic_encode

def main(request, response):
    """Respond to `/cookie/set/secure?{value}` by setting two cookies:
    alone_secure={value};secure;path=/`
    alone_insecure={value};path=/"""
    headers = setNoCacheAndCORSHeaders(request, response)
    value = isomorphic_encode(request.url_parts.query)

    headers.append(makeCookieHeader(b"alone_secure", value, {b"secure": b"", b"path": b"/"}))
    headers.append(makeCookieHeader(b"alone_insecure", value, {b"path": b"/"}))
    return headers, b'{"success": true}'
