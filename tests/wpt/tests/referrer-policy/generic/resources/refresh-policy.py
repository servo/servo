def main(request, _response):
    response_headers = [("Content-Type", "text/html")]

    body = "<!doctype html>"

    url = request.GET.first(b"url", b"").decode()
    if url:
        response_headers.append(("Refresh", "0; url=" + url))
        body += "Refreshing to %s" % url
    else:
        body += "Not refreshing"

    policy = request.GET.first(b"policy", b"").decode()
    response_headers.append(("Referrer-Policy", policy))

    return 200, response_headers, body
