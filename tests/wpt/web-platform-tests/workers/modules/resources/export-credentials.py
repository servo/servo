def main(request, response):
    cookie = request.cookies.first("COOKIE_NAME", None)

    response_headers = [("Content-Type", "text/javascript"),
                        ("Access-Control-Allow-Credentials", "true")]

    origin = request.headers.get("Origin", None)
    if origin:
        response_headers.append(("Access-Control-Allow-Origin", origin))

    cookie_value = '';
    if cookie:
        cookie_value = cookie.value;
    return (200, response_headers,
            "export const cookie = '"+cookie_value+"';")
