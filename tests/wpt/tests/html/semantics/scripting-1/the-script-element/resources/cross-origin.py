def main(request, response):
    origin = request.headers.get(b"origin")

    if origin is not None:
        response.headers.set(b"Access-Control-Allow-Origin", origin)
        response.headers.set(b"Access-Control-Allow-Methods", b"GET")
        response.headers.set(b"Access-Control-Allow-Credentials", b"true")

    if request.method == u"OPTIONS":
        return u""

    headers = [(b"Content-Type", b"text/javascript")]
    milk = request.cookies.first(b"milk", None)

    if milk is None:
        return headers, u"var included = false;"
    elif milk.value == b"yes":
        return headers, u"var included = true;"

    return headers, u"var included = false;"
