# Returns a valid response when a request has appropriate credentials.
def main(request, response):
    cookie = request.cookies.first("cookieName", None)
    expected_value = request.GET.first("value", None)
    source_origin = request.headers.get("origin", None)

    response_headers = [("Content-Type", "text/javascript"),
                        ("Access-Control-Allow-Origin", source_origin),
                        ("Access-Control-Allow-Credentials", "true")]

    if cookie == expected_value:
        return (200, response_headers, "")

    return (404, response_headers)
