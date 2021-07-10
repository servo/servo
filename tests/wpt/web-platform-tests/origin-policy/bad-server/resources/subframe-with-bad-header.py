def main(request, response):
    """Send a response with the Origin-Policy header given in the query string.
    """
    header = request.GET.first(b"header")

    response.headers.set(b"Origin-Policy", header)
    response.headers.set(b"Content-Type", b"text/html")

    return u"""
    <!DOCTYPE html>
    <meta charset="utf-8">
    <title>Origin policy bad header subframe</title>
    """
