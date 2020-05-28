def main(request, response):
    headers = [
        (b"Content-Type", b"text/javascript"),
        (b"Access-Control-Allow-Origin", request.headers.get(b"Origin")),
        (b"Access-Control-Allow-Credentials", b"true")
    ]
    identifier = request.GET.first(b"id")
    cookie_name = request.GET.first(b"cookieName")
    cookie = request.cookies.first(cookie_name, None)
    if identifier is None or cookie_name is None:
        return headers, b""

    if cookie is None:
        result = b"not found"
    elif cookie.value == b"1":
        result = b"found"
    else:
        result = b"different value: " + cookie.value

    return headers, b"window." + identifier + b" = '" + result + b"';"
