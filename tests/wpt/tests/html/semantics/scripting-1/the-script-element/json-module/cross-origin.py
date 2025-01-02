def main(request, response):

    headers = [
        (b"Content-Type", b"application/json"),
        (b"Access-Control-Allow-Origin", request.GET.first(b"origin")),
        (b"Access-Control-Allow-Credentials", b"true")
    ]

    milk = request.cookies.first(b"milk", None)

    if milk is None:
        return headers, u'{"requestHadCookies": false}'
    elif milk.value == b"1":
        return headers, u'{"requestHadCookies": true}'

    return headers, u'{"requestHadCookies": false}'
