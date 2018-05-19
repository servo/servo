from helpers import makeCookieHeader, readParameter, setNoCacheAndCORSHeaders

def main(request, response):
    """Respond to `/cookie/set/secure?{value}` by setting two cookies:
    alone_secure={value};secure;path=/`
    alone_insecure={value};path=/"""
    headers = setNoCacheAndCORSHeaders(request, response)
    value = request.url_parts.query

    headers.append(makeCookieHeader("alone_secure", value, {"secure": "","path": "/"}))
    headers.append(makeCookieHeader("alone_insecure", value, {"path": "/"}))
    return headers, '{"success": true}'
