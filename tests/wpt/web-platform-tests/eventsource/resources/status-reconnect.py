def main(request, response):
    status_code = request.GET.first("status", "204")
    name = request.GET.first("id", status_code)

    headers = [("Content-Type", "text/event-stream")]

    cookie_name = "request" + name

    if request.cookies.first(cookie_name, "") == status_code:
        status = 200
        response.delete_cookie(cookie_name)
        body = "data: data\n\n"
    else:
        response.set_cookie(cookie_name, status_code);
        status = (int(status_code), "TEST")
        body = "retry: 2\n"
        if "ok_first" in request.GET:
            body += "data: ok\n\n"

    return status, headers, body

