import datetime

def main(request, response):
    cookie_name = request.GET.first("cookie_name", "")

    response.headers.set("Cache-Control", "no-store")
    response.headers.set("Access-Control-Allow-Origin", request.headers.get("origin"))
    response.headers.set("Access-Control-Allow-Credentials", "true")

    for cookie in request.cookies:
        # Set cookie to expire yesterday
        response.set_cookie(cookie, "deleted", expires=-datetime.timedelta(days=1))

    if cookie_name:
        # Set cookie to expire tomorrow
        response.set_cookie(cookie_name, "COOKIE", expires=datetime.timedelta(days=1))
