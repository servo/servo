def main(request, response):
    name = b"recon_fail_" + request.GET.first(b"id")

    headers = [(b"Content-Type", b"text/event-stream")]
    cookie = request.cookies.first(name, None)
    state = cookie.value if cookie is not None else None

    if state == b'opened':
        status = (200, b"RECONNECT")
        response.set_cookie(name, b"reconnected");
        body = b"data: reconnected\n\n";

    elif state == b'reconnected':
        status = (204, b"NO CONTENT (CLOSE)")
        response.delete_cookie(name);
        body = b"data: closed\n\n" # Will never get through

    else:
        status = (200, b"OPEN");
        response.set_cookie(name, b"opened");
        body = b"retry: 2\ndata: opened\n\n";

    return status, headers, body

