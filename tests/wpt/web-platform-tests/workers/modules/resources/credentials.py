def main(request, response):
    cookie = request.cookies.first("COOKIE_NAME", None)

    response_headers = [("Content-Type", "text/javascript"),
                        ("Access-Control-Allow-Credentials", "true")]

    cookie_value = '';
    if cookie:
        cookie_value = cookie.value;
    return (200, response_headers, "postMessage('"+cookie_value+"');")
