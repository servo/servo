def verify_authenticated(request, response):
    expected_cookie_name_and_value = request.body
    if expected_cookie_name_and_value == b"":
        expected_cookie_name_and_value = b"auth_cookie=abcdef0123"
    (expected_name, expected_value) = expected_cookie_name_and_value.split(b"=")

    headers = []
    # Only CORS requests need the CORS headers
    if request.headers.get(b"origin") != None:
      headers = [(b"Access-Control-Allow-Origin",request.headers.get(b"origin")),
                 (b"Access-Control-Allow-Credentials", b"true")]

    cookie = request.cookies.get(expected_name)
    if cookie == None or cookie.value != expected_value:
        return (403, headers, "")
    return (200, headers, "")
