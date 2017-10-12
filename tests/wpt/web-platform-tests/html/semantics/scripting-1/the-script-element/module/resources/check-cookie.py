def main(request, response):
    headers = [
        ("Content-Type", "text/javascript"),
        ("Access-Control-Allow-Origin", request.headers.get("Origin")),
        ("Access-Control-Allow-Credentials", "true")
    ]
    identifier = request.GET.first("id")
    cookie_name = request.GET.first("cookieName")
    cookie = request.cookies.first(cookie_name, None)
    if identifier is None or cookie_name is None:
        return headers, ""

    if cookie is None:
        result = "not found"
    elif cookie.value == "1":
        result = "found"
    else:
        result = "different value: " + cookie.value

    return headers, "window." + identifier + " = '" + result + "';"
