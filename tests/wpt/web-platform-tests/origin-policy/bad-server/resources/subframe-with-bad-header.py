def main(request, response):
    """Send a response with the Origin-Policy header given in the query string.
    """
    header = request.GET.first("header")

    response.headers.set("Origin-Policy", header)
    response.headers.set("Content-Type", "text/html")

    return """
    <!DOCTYPE html>
    <meta charset="utf-8">
    <title>Origin policy bad header subframe</title>
    """
