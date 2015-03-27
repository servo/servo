def main(request, response):
    name = "recon_fail_" + request.GET.first("id")

    headers = [("Content-Type", "text/event-stream")]
    cookie = request.cookies.first(name, None)
    state = cookie.value if cookie is not None else None

    if state == 'opened':
        status = (200, "RECONNECT")
        response.set_cookie(name, "reconnected");
        body = "data: reconnected\n\n";

    elif state == 'reconnected':
        status = (204, "NO CONTENT (CLOSE)")
        response.delete_cookie(name);
        body = "data: closed\n\n" # Will never get through

    else:
        status = (200, "OPEN");
        response.set_cookie(name, "opened");
        body = "retry: 2\ndata: opened\n\n";

    return status, headers, body

