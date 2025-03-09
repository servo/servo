def main(request, response):
    expected_cookie_name_and_value = request.body
    if expected_cookie_name_and_value == b"":
        expected_cookie_name_and_value = b"auth_cookie=abcdef0123"
    (expected_name, expected_value) = expected_cookie_name_and_value.split(b"=")

    cookie = request.cookies.get(expected_name)
    if cookie == None or cookie.value != expected_value:
        return (401, response.headers, "")
    return (200, response.headers, "")
