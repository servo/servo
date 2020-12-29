import json

from cookies.resources.helpers import setNoCacheAndCORSHeaders
from wptserve.utils import isomorphic_decode
from wptserve.utils import isomorphic_encode

def set_cookie(headers, cookie_string, drop=False):
    """Helper method to add a Set-Cookie header"""
    if drop:
        cookie_string = cookie_string.encode('utf-8') + b'; max-age=0'
    headers.append((b'Set-Cookie', isomorphic_encode(cookie_string)))

def main(request, response):
    """Set or drop a cookie via GET params.

    Usage: `/cookie.py?set={cookie}` or `/cookie.py?drop={cookie}`

    The passed-in cookie string should be stringified via JSON.stringify() (in
    the case of multiple cookie headers sent in an array) and encoded via
    encodeURIComponent, otherwise `parse_qsl` will split on any semicolons
    (used by the Request.GET property getter). Note that values returned by
    Request.GET will decode any percent-encoded sequences sent in a GET param
    (which may or may not be surprising depending on what you're doing).

    Note: here we don't use Response.delete_cookie() or similar other methods
    in this resources directory because there are edge cases that are impossible
    to express via those APIs, namely a bare (`Path`) or empty Path (`Path=`)
    attribute. Instead, we pipe through the entire cookie and append `max-age=0`
    to it.
    """
    headers = setNoCacheAndCORSHeaders(request, response)

    try:
        if b'drop' in request.GET:
            cookie = request.GET[b'drop']
            cookie = json.loads(cookie)
            cookies = cookie if isinstance(cookie, list) else [cookie]
            for c in cookies:
                set_cookie(headers, c, drop=True)

        if b'set' in request.GET:
            cookie = isomorphic_decode(request.GET[b'set'])
            cookie = json.loads(cookie)
            cookies = cookie if isinstance(cookie, list) else [cookie]
            for c in cookies:
                set_cookie(headers, c)

        if b'location' in request.GET:
            headers.append((b'Location', request.GET[b'location']))
            return 302, headers, b'{"redirect": true}'

        return headers, b'{"success": true}'
    except Exception as e:
          return 500, headers, bytes({'error': '{}'.format(e)})
