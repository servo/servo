import datetime

def main(request, response):
    cookie_name = request.GET.first(b"cookie_name", b"")

    response.headers.set(b"Cache-Control", b"no-store")
    response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))
    response.headers.set(b"Access-Control-Allow-Credentials", b"true")

    for cookie in request.cookies:
        # Set cookie to expire yesterday
        response.set_cookie(cookie, b"deleted", expires=-datetime.timedelta(days=1))

    if cookie_name:
        # Set cookie to expire tomorrow
        response.set_cookie(cookie_name, b"COOKIE", expires=datetime.timedelta(days=1))
