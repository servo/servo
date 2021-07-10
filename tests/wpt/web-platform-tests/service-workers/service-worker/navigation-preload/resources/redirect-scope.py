def main(request, response):
    if b"base" in request.GET:
        return [(b"Content-Type", b"text/html")], b"OK"
    type = request.GET.first(b"type")

    if type == b"normal":
        response.status = 302
        response.headers.append(b"Location", b"redirect-redirected.html")
        response.headers.append(b"Custom-Header", b"hello")
        return b""

    if type == b"no-location":
        response.status = 302
        response.headers.append(b"Custom-Header", b"hello")
        return b""

    if type == b"no-location-with-body":
        response.status = 302
        response.headers.append(b"Content-Type", b"text/html")
        response.headers.append(b"Custom-Header", b"hello")
        return b"<body>BODY</body>"

    if type == b"redirect-to-scope":
        response.status = 302
        response.headers.append(b"Location",
                                b"redirect-scope.py?type=redirect-to-scope2")
        return b""
    if type == b"redirect-to-scope2":
        response.status = 302
        response.headers.append(b"Location",
                                b"redirect-scope.py?type=redirect-to-scope3")
        return b""
    if type == b"redirect-to-scope3":
        response.status = 302
        response.headers.append(b"Location", b"redirect-redirected.html")
        response.headers.append(b"Custom-Header", b"hello")
        return b""
