def main(request, response):
    """
    Returns a response with a Set-Cookie header based on the query params.
    The body will be "1" if the cookie is present in the request and `drop` parameter is "0",
    otherwise the body will be "0".
    """
    same_site = request.GET.first(b"same-site")
    cookie_name = request.GET.first(b"cookie-name")
    drop = request.GET.first(b"drop")
    cookie_in_request = b"0"
    cookie = b"%s=1; Secure; SameSite=%s" % (cookie_name, same_site)

    if drop == b"1":
        cookie += b"; Max-Age=0"

    if request.cookies.get(cookie_name):
        cookie_in_request = request.cookies[cookie_name].value

    headers = [(b'Content-Type', b'text/html'), (b'Set-Cookie', cookie)]
    return (200, headers, cookie_in_request)
