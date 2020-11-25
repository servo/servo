from cookies.resources.helpers import setNoCacheAndCORSHeaders
from wptserve.utils import isomorphic_encode

def main(request, response):
    """Set or drop a cookie via GET params.

    Usage: `/cookie.py?set={cookie}` or `/cookie.py?drop={cookie}`

    The passed-in cookie string should be encoded via encodeURIComponent,
    otherwise `parse_qsl` will split on any semicolons (used by the Request.GET
    property getter).

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
            cookie += "; max-age=0"

        if b'set' in request.GET:
            cookie = request.GET[b'set']

        headers.append((b'Set-Cookie', isomorphic_encode(cookie)))
        return headers, b'{"success": true}'
    except Exception as e:
          return 500, headers, bytes({'error': '{}'.format(e)})

