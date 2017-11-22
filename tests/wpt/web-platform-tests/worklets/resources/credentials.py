# Returns a valid response when a request has appropriate credentials.
def main(request, response):
    credentials_mode = request.GET.first("mode")
    cookie = request.cookies.first("cookieName", None)
    source_origin = request.headers.get("origin", None);
    is_cross_origin = request.GET.first("is_cross_origin", False)

    # The request with the default WorkletOptions should not include the cookie.
    if credentials_mode is "default" and cookie is not None:
        return (404)

    # The request with "credentials=omit" should not include the cookie.
    if credentials_mode is "omit" and cookie is not None:
        return (404)

    if credentials_mode is "same-origin":
        # The cross-origin request with "credentials=same-origin" should not
        # include the cookie.
        if is_cross_origin and cookie is not None:
          return (404)
        # The same-origin request with "credentials=same-origin" should include
        # the cookie.
        if not is_cross_origin and cookie is None:
          return (404)

    # The request with "credentials=include" should include the cookie.
    if credentials_mode is "include" and cookie is None:
        return (404)

    return (200, [("Content-Type", "text/javascript"),
                  ("Access-Control-Allow-Origin", source_origin),
                  ("Access-Control-Allow-Credentials", "true")], "")
