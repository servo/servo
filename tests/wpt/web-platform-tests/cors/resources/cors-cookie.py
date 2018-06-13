
def main(request, response):
    origin = request.GET.first("origin", request.headers["origin"])
    credentials = request.GET.first("credentials", "true")

    headers = [("Content-Type", "text/plain")]
    if origin != 'none':
        headers.append(("Access-Control-Allow-Origin", origin))
    if credentials != 'none':
        headers.append(("Access-Control-Allow-Credentials", credentials))

    ident = request.GET.first('ident', 'test')

    if ident in request.cookies:
        body = request.cookies[ident].value
        response.delete_cookie(ident)
    else:
        response.set_cookie(ident, "COOKIE")
        body = "NO_COOKIE"

    return headers, body
