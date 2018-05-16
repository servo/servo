from helpers import makeCookieHeader, readParameter, setNoCacheAndCORSHeaders

def main(request, response):
    """Respond to `/cookie/set/samesite?{value}` by setting three cookies:
    1. `samesite_strict={value};SameSite=Strict;path=/`
    2. `samesite_lax={value};SameSite=Lax;path=/`
    3. `samesite_none={value};path=/`"""
    headers = setNoCacheAndCORSHeaders(request, response)
    value = request.url_parts.query

    headers.append(makeCookieHeader("samesite_strict", value, {"SameSite":"Strict","path":"/"}))
    headers.append(makeCookieHeader("samesite_lax", value, {"SameSite":"Lax","path":"/"}))
    headers.append(makeCookieHeader("samesite_none", value, {"path":"/"}))
    return headers, '{"success": true}'
