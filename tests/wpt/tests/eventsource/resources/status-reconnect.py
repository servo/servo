def main(request, response):
    status_code = request.GET.first(b"status", b"204")
    name = request.GET.first(b"id", status_code)

    headers = [(b"Content-Type", b"text/event-stream")]

    cookie_name = b"request" + name

    if request.cookies.first(cookie_name, b"") == status_code:
        status = 200
        response.delete_cookie(cookie_name)
        body = b"data: data\n\n"
    else:
        response.set_cookie(cookie_name, status_code);
        status = (int(status_code), b"TEST")
        body = b"retry: 2\n"
        if b"ok_first" in request.GET:
            body += b"data: ok\n\n"

    return status, headers, body

