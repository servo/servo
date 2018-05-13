import urlparse

def setNoCacheAndCORSHeaders(request, response):
    """Set Cache-Control, CORS and Content-Type headers appropriate for the cookie tests."""
    headers = [("Content-Type", "application/json"),
               ("Access-Control-Allow-Credentials", "true")]

    origin = "*"
    if "origin" in request.headers:
        origin = request.headers["origin"]

    headers.append(("Access-Control-Allow-Origin", origin))
    #headers.append(("Access-Control-Allow-Credentials", "true"))
    headers.append(("Cache-Control", "no-cache"))
    headers.append(("Expires", "Fri, 01 Jan 1990 00:00:00 GMT"))

    return headers

def makeCookieHeader(name, value, otherAttrs):
    """Make a Set-Cookie header for a cookie with the name, value and attributes provided."""
    def makeAV(a, v):
        if None == v or "" == v:
            return a
        return "%s=%s" % (a, v)

    # ensure cookie name is always first
    attrs = ["%s=%s" % (name, value)]
    attrs.extend(makeAV(a, v) for (a,v) in otherAttrs.iteritems())
    return ("Set-Cookie", "; ".join(attrs))

def makeDropCookie(name, secure):
    attrs = {"MaxAge": 0, "path": "/"}
    if secure:
        attrs["secure"] = ""
    return makeCookieHeader(name, "", attrs)

def readParameter(request, paramName, requireValue):
    """Read a parameter from the request. Raise if requireValue is set and the
    parameter has an empty value or is not present."""
    params = urlparse.parse_qs(request.url_parts.query)
    param = params[paramName][0].strip()
    if len(param) == 0:
        raise Exception("Empty or missing name parameter.")
    return param

def readCookies(request):
    """Read the cookies from the client present in the request."""
    cookies = {}
    for key in request.cookies:
        for cookie in request.cookies.get_list(key):
            # do we care we'll clobber cookies here? If so, do we
            # need to modify the test to take cookie names and value lists?
            cookies[key] = cookie.value
    return cookies

