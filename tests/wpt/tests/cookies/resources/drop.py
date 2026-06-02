from cookies.resources.helpers import makeDropCookie, readParameter, setNoCacheAndCORSHeaders

def main(request, response):
    """Respond to `/cookie/drop?name={name}` by expiring the cookie named `{name}`."""
    headers = setNoCacheAndCORSHeaders(request, response)
    try:
        # Expire the named cookie, and return a JSON-encoded success code.
        name = readParameter(request, paramName=u"name", requireValue=True)
        scheme = request.url_parts.scheme
        headers.append(makeDropCookie(name, u"https" == scheme))
        return headers, b'{"success": true}'
    except:
        return 500, headers, b'{"error" : "Empty or missing name parameter."}'

