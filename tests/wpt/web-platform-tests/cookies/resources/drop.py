from helpers import makeDropCookie, readParameter, setNoCacheAndCORSHeaders

def main(request, response):
    """Respond to `/cookie/drop?name={name}` by expiring the cookie named `{name}`."""
    headers = setNoCacheAndCORSHeaders(request, response)
    try:
        # Expire the named cookie, and return a JSON-encoded success code.
        name = readParameter(request, paramName="name", requireValue=True)
        scheme = request.url_parts.scheme
        headers.append(makeDropCookie(name,  "https" == scheme))
        return headers, '{"success": true}'
    except:
        return 500, headers, '{"error" : "Empty or missing name parameter."}'


