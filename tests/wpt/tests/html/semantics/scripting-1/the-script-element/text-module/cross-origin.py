def main(request, response):

    headers = [
        (b"Content-Type", b"text/plain"),
        (b"Access-Control-Allow-Origin", request.GET.first(b"origin")),
        (b"Access-Control-Allow-Credentials", b"true")
    ]

    milk = request.cookies.first(b"milk", None)

    if milk is not None and milk.value == b"1":
        return headers, u'request had cookies'
    else:
        return headers, u'request did not have cookies'
