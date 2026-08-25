def main(request, response):

    headers = [
        (b"Content-Type", b"text/css"),
        (b"Access-Control-Allow-Origin", request.GET.first(b"origin")),
        (b"Access-Control-Allow-Credentials", b"true")
    ]

    milk = request.cookies.first(b"milk", None)

    # Send back
    if milk is None:
        return headers, u'.requestDidNotHaveCookies { }'
    elif milk.value == b"1":
        return headers, u'.requestHadCookies { }'

    return headers, u'.requestDidNotHaveCookies { }'
