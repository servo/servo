from helpers import makeCookieHeader, setNoCacheAndCORSHeaders

def main(request, response):
    """Respond to `/cookies/resources/setSameSiteNone.py?{value}` by setting two cookies:
    1. `samesite_none_insecure={value};SameSite=None;path=/`
    2. `samesite_none_secure={value};SameSite=None;Secure;path=/`
    """
    headers = setNoCacheAndCORSHeaders(request, response)
    value = request.url_parts.query

    headers.append(makeCookieHeader("samesite_none_insecure", value, {"SameSite":"None", "path":"/"}))
    headers.append(makeCookieHeader("samesite_none_secure", value, {"SameSite":"None", "Secure":"", "path":"/"}))

    return headers, '{"success": true}'
