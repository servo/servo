def main(request, response):
    """
    Reflect Sec-Ch-Viewport-Width and Sec-Ch-Viewport-Height headers
    """

    if b"sec-ch-viewport-width" in request.headers and b"sec-ch-viewport-height" in request.headers:
        result = request.headers["sec-ch-viewport-width"] + b"," + request.headers["sec-ch-viewport-height"]
    else:
        result = u"FAIL"

    headers = [(b"Content-Type", b"text/html"),
               (b"Access-Control-Allow-Origin", b"*")]
    return 200, headers, result
