def main(request, response):
    """
    Returns a response with a Set-Cookie header based on the query params.
    The body will be "1" if the cookie is present in the request and `drop` parameter is "0",
    otherwise the body will be "0".
    """
    same_site = request.GET.first("same-site")
    cookie_name = request.GET.first("cookie-name")
    drop = request.GET.first("drop")
    cookie_in_request = "0"
    cookie = "%s=1; Secure; SameSite=%s" % (cookie_name, same_site)

    if drop == "1":
        cookie += "; Max-Age=0"

    if request.cookies.get(cookie_name):
        cookie_in_request = request.cookies[cookie_name].value

    headers = [('Content-Type', 'text/html'), ('Set-Cookie', cookie)]
    return (200, headers, cookie_in_request)
