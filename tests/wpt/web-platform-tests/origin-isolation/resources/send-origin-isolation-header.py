def main(request, response):
    """Send a response with the Origin-Isolation header given in the "header"
    query parameter, or no header if that is not provided. In either case, the
    response will listen for message and messageerror events and echo them back
    to the parent. See ./helpers.mjs for how these handlers are used.
    """

    if b"header" in request.GET:
      header = request.GET.first(b"header")
      response.headers.set(b"Origin-Isolation", header)

    response.headers.set(b"Content-Type", b"text/html")

    return u"""
    <!DOCTYPE html>
    <meta charset="utf-8">
    <title>Helper page for origin isolation tests</title>

    <body>
    <script type="module" src="child-frame-script.mjs"></script>
    """
