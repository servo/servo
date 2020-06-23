def main(request, response):
    """Send a response with the Origin-Isolation header given in the "header"
    query parameter, or no header if that is not provided. In either case, the
    response will listen for message and messageerror events and echo them back
    to the parent. See ./helpers.mjs for how these handlers are used.
    """

    if "header" in request.GET:
      header = request.GET.first("header")
      response.headers.set("Origin-Isolation", header)

    response.headers.set("Content-Type", "text/html")

    return """
    <!DOCTYPE html>
    <meta charset="utf-8">
    <title>Helper page for origin isolation tests</title>

    <body>
    <script type="module" src="child-frame-script.mjs"></script>
    """
